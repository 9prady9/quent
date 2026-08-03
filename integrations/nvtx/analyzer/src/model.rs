// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The two-pass reconstruction entry point.
//!
//! Pass 1 materializes the stream in timestamp order and builds the
//! handle-resolution tables; pass 2 replays it against them. Splitting the two
//! is what makes replay tolerant of arrival order in both dimensions — by then
//! neither an end before its start nor a `RegisterString` after the range using
//! it is still possible.

use std::collections::BTreeMap;

use nvtx_bridge::NvtxEventEntity;
use nvtx_events::NvtxEvent;
use quent_events::Event;
use quent_time::{TimeOrderedCollector, TimeUnixNanoSec};

use crate::ranges::{PushPopRanges, StartEndRanges};
use crate::resource::Resources;
use crate::span::{
    NvtxCategory, NvtxDomain, NvtxMark, NvtxSpan, NvtxThread, SpanId, SpanKind, category_id,
};
use crate::stats::{self, RangeStats, StatsKey};
use crate::tables::ResolutionTables;

/// An in-memory model reconstructed from a captured NVTX event stream.
#[derive(Debug, Default)]
pub struct NvtxModel {
    spans: Vec<NvtxSpan>,
    marks: Vec<NvtxMark>,
    domains: Vec<NvtxDomain>,
    threads: Vec<NvtxThread>,
    categories: Vec<NvtxCategory>,
    /// Retained so names not attached to a reconstructed entity — an unnamed
    /// thread, a category referenced by nothing yet — still resolve on demand.
    tables: ResolutionTables,
}

impl NvtxModel {
    /// Every reconstructed span.
    ///
    /// A span's index here *is* its [`SpanId`], which is what makes
    /// [`NvtxSpan::parent`] resolvable. Ordering is by opening: push/pop ranges
    /// take their slot when pushed, start/end ranges when they close.
    pub fn spans(&self) -> &[NvtxSpan] {
        &self.spans
    }

    /// The span a [`SpanId`] refers to, or `None` if it came from another model.
    pub fn span(&self, id: SpanId) -> Option<&NvtxSpan> {
        self.spans.get(id.0)
    }

    /// Every reconstructed mark, in timestamp order.
    pub fn marks(&self) -> &[NvtxMark] {
        &self.marks
    }

    /// Every reconstructed resource lifespan: the interval between a create and
    /// its destroy.
    ///
    /// NVTX says only that the handle existed and what it was called, so nothing
    /// about its size or occupancy is inferred.
    pub fn resources(&self) -> impl Iterator<Item = &NvtxSpan> {
        self.spans
            .iter()
            .filter(|span| span.kind == SpanKind::Resource)
    }

    /// Every domain the stream mentioned, ordered by raw handle.
    pub fn domains(&self) -> &[NvtxDomain] {
        &self.domains
    }

    /// Every OS thread the stream mentioned, ordered by id.
    pub fn threads(&self) -> &[NvtxThread] {
        &self.threads
    }

    /// Every non-zero category the stream mentioned, ordered by `(domain, id)`.
    pub fn categories(&self) -> &[NvtxCategory] {
        &self.categories
    }

    /// Aggregated durations per `(name, domain, category)`.
    ///
    /// Range spans only. Marks have no duration and resource lifespans measure
    /// existence rather than work, so neither participates.
    pub fn range_statistics(&self) -> BTreeMap<StatsKey, RangeStats> {
        stats::range_statistics(&self.spans)
    }

    /// The resolved name of a category *within its domain*.
    ///
    /// The domain is required, not optional: NVTX category ids are unique only
    /// within a domain, so resolving one globally would return another domain's
    /// name. Returns `None` for category `0`, NVTX's "no category" sentinel.
    pub fn category_name(&self, domain: u64, category: u32) -> Option<String> {
        self.tables.resolve_category(domain, category)
    }

    /// The resolved name of an OS thread.
    ///
    /// Threads are usually unnamed, so this answers for *any* id — a thread the
    /// stream never named renders as `"thread {id}"` rather than nothing.
    pub fn thread_name(&self, thread_id: u32) -> String {
        self.tables.resolve_thread(thread_id)
    }
}

/// Write a closed span into the slot reserved for it when it was pushed.
///
/// Bounds-checked rather than indexed: a stray id is dropped, not a panic.
fn fill(slots: &mut [Option<NvtxSpan>], id: SpanId, span: NvtxSpan) {
    if let Some(slot) = slots.get_mut(id.0) {
        *slot = Some(span);
    }
}

/// Builds an [`NvtxModel`] from a captured event stream.
#[derive(Debug, Default)]
pub struct NvtxModelBuilder;

impl NvtxModelBuilder {
    /// Reconstruct a model from a captured NVTX event stream.
    ///
    /// Tolerant by construction: out-of-order events are reordered, forward
    /// references to not-yet-registered handles resolve anyway, duplicate
    /// timestamps reconstruct deterministically, ranges left open at the end of
    /// the stream are closed and flagged, and closes with no matching open are
    /// logged and skipped. Handles that never resolve get a stable placeholder.
    /// No malformed stream aborts the build or panics.
    ///
    /// Infallible by signature: there is no anomaly a caller could be asked to
    /// handle. `build` takes events already decoded, so a decode failure
    /// belongs to whichever crate does the decoding.
    pub fn build(events: impl IntoIterator<Item = Event<NvtxEventEntity>>) -> NvtxModel {
        // Pass 1a — materialize in timestamp order. Equal timestamps keep
        // arrival order, so replay is deterministic.
        let mut collector = TimeOrderedCollector::default();
        collector.extend(events);
        let ordered = collector.into_inner();

        // Pass 1b — learn every name in the stream before resolving any of them.
        let tables = ResolutionTables::build(&ordered);

        // Pass 2 — replay.
        let mut ranges = StartEndRanges::default();
        let mut pushes = PushPopRanges::default();
        let mut resources = Resources::default();
        let mut marks = Vec::new();
        let mut trace_end: TimeUnixNanoSec = 0;

        // Slots, not a plain span list: a `SpanId` is an index here, and a
        // nested push needs its parent's id at pop time — while the parent is
        // still open. Reserving the slot at push time makes that id exist
        // before the span does.
        let mut slots: Vec<Option<NvtxSpan>> = Vec::new();

        for event in ordered {
            trace_end = trace_end.max(event.timestamp);

            match event.data.0 {
                NvtxEvent::RangeStart {
                    domain,
                    range_id,
                    attributes,
                } => {
                    let name = tables.resolve_message(domain, &attributes.message);
                    if let Some(span) =
                        ranges.start(range_id, domain, name, attributes, event.timestamp)
                    {
                        slots.push(Some(span));
                    }
                }
                // The domain on a `RangeEnd` is redundant: `range_id` is
                // process-globally unique, so it alone identifies the range.
                NvtxEvent::RangeEnd { range_id, .. } => {
                    if let Some(span) = ranges.end(range_id, event.timestamp) {
                        slots.push(Some(span));
                    }
                }
                NvtxEvent::RangePush {
                    domain,
                    thread_id,
                    attributes,
                } => {
                    let name = tables.resolve_message(domain, &attributes.message);
                    let id = SpanId(slots.len());
                    slots.push(None);
                    pushes.push(id, thread_id, domain, name, attributes, event.timestamp);
                }
                NvtxEvent::RangePop { domain, thread_id } => {
                    if let Some((id, span)) = pushes.pop(thread_id, domain, event.timestamp) {
                        fill(&mut slots, id, span);
                    }
                }
                NvtxEvent::Mark { domain, attributes } => marks.push(NvtxMark {
                    domain,
                    // `nvtxDomainMarkEx` carries no thread id.
                    thread_id: None,
                    name: tables.resolve_message(domain, &attributes.message),
                    category: category_id(attributes.category),
                    color: attributes.color,
                    payload: attributes.payload,
                    timestamp: event.timestamp,
                }),
                NvtxEvent::ResourceCreate {
                    domain,
                    handle,
                    identifier_type,
                    // Interpreting the raw identifier bits depends on
                    // `identifier_type`; decoding extension classes is out of
                    // scope for this crate.
                    identifier: _,
                    message,
                } => {
                    let name = tables.resolve_message(domain, &message);
                    if let Some(span) =
                        resources.create(handle, domain, name, identifier_type, event.timestamp)
                    {
                        slots.push(Some(span));
                    }
                }
                // Matched on `handle` alone — the event carries no domain, so
                // there is nothing else to key on.
                NvtxEvent::ResourceDestroy { handle } => {
                    if let Some(span) = resources.destroy(handle, event.timestamp) {
                        slots.push(Some(span));
                    }
                }
                // Consumed by pass 1; they only advance the trace end here.
                // Listed explicitly rather than caught by `_`, so a new
                // `NvtxEvent` variant fails to compile instead of being
                // silently discarded.
                NvtxEvent::DomainCreate { .. }
                | NvtxEvent::DomainDestroy { .. }
                | NvtxEvent::RegisterString { .. }
                | NvtxEvent::NameCategory { .. }
                | NvtxEvent::NameThread { .. } => {}
            }
        }

        // Anything still open never had its close observed.
        slots.extend(ranges.close_at_trace_end(trace_end).into_iter().map(Some));
        slots.extend(
            resources
                .close_at_trace_end(trace_end)
                .into_iter()
                .map(Some),
        );
        for (id, span) in pushes.close_at_trace_end(trace_end) {
            fill(&mut slots, id, span);
        }

        // Flattening preserves the indices the `SpanId`s were handed out
        // against only while every slot is filled; a hole would shift every
        // later index, so the invariant is asserted rather than assumed.
        debug_assert!(
            slots.iter().all(Option::is_some),
            "an unfilled slot would invalidate every SpanId after it"
        );
        // `Flatten` reports a lower size hint of `0`, so collecting alone would
        // grow the span list by repeated doubling.
        let mut spans: Vec<NvtxSpan> = Vec::with_capacity(slots.len());
        spans.extend(slots.into_iter().flatten());

        let domains = tables.domain_records();
        let threads = tables.thread_records();
        let categories = tables.category_records();

        NvtxModel {
            spans,
            marks,
            domains,
            threads,
            categories,
            tables,
        }
    }
}

// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Reconstruction of NVTX ranges — two kinds, two match keys.
//!
//! `RangeStart`/`RangeEnd` correlate by `range_id` alone: NVTX assigns ids from
//! one process-global counter, so they match across threads and domains and the
//! domain on a `RangeEnd` is redundant.
//!
//! `RangePush`/`RangePop` carry no id and correlate by position on a stack, one
//! stack per `(thread_id, domain)`. That grain mirrors the injection layer's own
//! `RANGE_DEPTH`, a thread-local keyed by domain. Keying coarser does not fail
//! loudly on a multi-threaded stream — it silently closes another thread's push
//! and reconstructs plausible-but-wrong nesting.

use std::collections::HashMap;

use nvtx_events::{NvtxColor, NvtxEventAttributes, NvtxPayload};
use quent_time::TimeUnixNanoSec;
use tracing::{debug, warn};

use crate::span::{NvtxSpan, SpanId, SpanKind, category_id};

/// An open range of either kind, awaiting whatever closes it.
///
/// Both kinds reconstruct the same span, so the rules that must not drift
/// between them — the `end >= start` clamp, the category sentinel, the
/// resource-only fields left empty — live here once.
struct OpenRange {
    domain: u64,
    /// `None` for `RangeStart`/`RangeEnd`, which are process-wide and carry no
    /// thread id.
    thread_id: Option<u32>,
    name: String,
    /// Only the attributes a span keeps. Retaining the whole
    /// `NvtxEventAttributes` would hold its `message` alive for as long as the
    /// range stays open, and the message is dead once `name` is resolved.
    category: Option<u32>,
    color: Option<NvtxColor>,
    payload: Option<NvtxPayload>,
    start: TimeUnixNanoSec,
    kind: SpanKind,
}

impl OpenRange {
    fn new(
        domain: u64,
        thread_id: Option<u32>,
        name: String,
        attributes: NvtxEventAttributes,
        start: TimeUnixNanoSec,
        kind: SpanKind,
    ) -> Self {
        Self {
            domain,
            thread_id,
            name,
            category: category_id(attributes.category),
            color: attributes.color,
            payload: attributes.payload,
            start,
            kind,
        }
    }

    /// Close this range, clamping `end` up to `start` so an inverted pair
    /// yields a zero-duration span rather than an underflowing one.
    fn close(self, end: TimeUnixNanoSec, parent: Option<SpanId>, synthetic_end: bool) -> NvtxSpan {
        NvtxSpan {
            domain: self.domain,
            thread_id: self.thread_id,
            name: self.name,
            category: self.category,
            color: self.color,
            payload: self.payload,
            start: self.start,
            end: end.max(self.start),
            kind: self.kind,
            // Only resource spans identify a thing; a range identifies work.
            identifier_type_label: None,
            parent,
            synthetic_end,
        }
    }
}

/// The set of currently-open process-wide ranges.
#[derive(Default)]
pub(crate) struct StartEndRanges {
    open: HashMap<u64, OpenRange>,
}

impl StartEndRanges {
    /// Record a `RangeStart` under its already-resolved `name`.
    ///
    /// Returns a span when this start *displaces* one still open under the same
    /// id. That reuse is malformed, but the displaced start was observed, so it
    /// is closed at the restart and flagged synthetic rather than erased.
    pub(crate) fn start(
        &mut self,
        range_id: u64,
        domain: u64,
        name: String,
        attributes: NvtxEventAttributes,
        start: TimeUnixNanoSec,
    ) -> Option<NvtxSpan> {
        let open = OpenRange::new(domain, None, name, attributes, start, SpanKind::StartEnd);
        let displaced = self.open.insert(range_id, open)?;
        warn!(
            "nvtx range id 0x{range_id:X} was restarted before it ended; closing the earlier \
             start at the restart ({start})"
        );
        Some(displaced.close(start, None, true))
    }

    /// Close the range matching `range_id`, if one is open.
    ///
    /// Returns `None` for an orphan end — skipped, never fatal.
    pub(crate) fn end(&mut self, range_id: u64, end: TimeUnixNanoSec) -> Option<NvtxSpan> {
        let Some(open) = self.open.remove(&range_id) else {
            // Routine, not anomalous: a range that started before capture
            // attached always ends without a recorded start.
            debug!("orphan nvtx range end for id 0x{range_id:X} with no open start; skipping");
            return None;
        };
        Some(open.close(end, None, false))
    }

    /// Close every range still open at the end of the trace.
    ///
    /// Ordered by `(start, range_id)` so several leaked ranges still
    /// reconstruct deterministically out of an unordered `HashMap`.
    pub(crate) fn close_at_trace_end(&mut self, trace_end: TimeUnixNanoSec) -> Vec<NvtxSpan> {
        let mut leaked: Vec<(u64, OpenRange)> = self.open.drain().collect();
        leaked.sort_by_key(|(range_id, open)| (open.start, *range_id));

        // One `warn!` for the count, per-range detail at `debug!`: a stream that
        // leaks thousands of ranges would otherwise emit thousands of warnings.
        if !leaked.is_empty() {
            warn!(
                "{} nvtx range(s) were never ended; closing them at trace end ({trace_end})",
                leaked.len()
            );
        }

        leaked
            .into_iter()
            .map(|(range_id, open)| {
                debug!("nvtx range id 0x{range_id:X} was never ended");
                open.close(trace_end, None, true)
            })
            .collect()
    }
}

/// The stack a push/pop range nests on: one OS thread within one domain.
///
/// The thread is explicit because the analyzer replays every captured thread's
/// events on one thread of its own.
type StackKey = (u32, u64);

/// A `RangePush` awaiting its matching `RangePop`.
struct OpenPushSpan {
    /// The slot reserved for this span at *push* time, because a child pops
    /// before its parent and needs the parent's id to record `parent`.
    id: SpanId,
    range: OpenRange,
}

impl OpenPushSpan {
    fn close(
        self,
        end: TimeUnixNanoSec,
        parent: Option<SpanId>,
        synthetic_end: bool,
    ) -> (SpanId, NvtxSpan) {
        (self.id, self.range.close(end, parent, synthetic_end))
    }
}

/// The currently-open push/pop ranges, one stack per `(thread_id, domain)`.
#[derive(Default)]
pub(crate) struct PushPopRanges {
    stacks: HashMap<StackKey, Vec<OpenPushSpan>>,
}

impl PushPopRanges {
    /// Record a `RangePush` under its already-resolved `name`, in the slot `id`.
    pub(crate) fn push(
        &mut self,
        id: SpanId,
        thread_id: u32,
        domain: u64,
        name: String,
        attributes: NvtxEventAttributes,
        start: TimeUnixNanoSec,
    ) {
        self.stacks
            .entry((thread_id, domain))
            .or_default()
            .push(OpenPushSpan {
                id,
                range: OpenRange::new(
                    domain,
                    Some(thread_id),
                    name,
                    attributes,
                    start,
                    SpanKind::PushPop,
                ),
            });
    }

    /// Close the innermost open push on `(thread_id, domain)`, if there is one.
    ///
    /// `parent` is read *after* the pop, so it is whatever encloses the range
    /// just closed — `None` when that range was outermost on its thread.
    /// Returns `None` for an orphan pop, logged and skipped.
    pub(crate) fn pop(
        &mut self,
        thread_id: u32,
        domain: u64,
        end: TimeUnixNanoSec,
    ) -> Option<(SpanId, NvtxSpan)> {
        let key = (thread_id, domain);
        // One lookup on the hot path: `pop` runs once per `RangePop`, roughly
        // half a push/pop-heavy stream. Borrowing the stack once answers what
        // closed, what encloses it, and whether the key is now empty.
        let popped = self.stacks.get_mut(&key).and_then(|stack| {
            let open = stack.pop()?;
            let parent = stack.last().map(|enclosing| enclosing.id);
            Some((open, parent, stack.is_empty()))
        });

        let Some((open, parent, now_empty)) = popped else {
            // Routine, like an orphan `RangeEnd`: a push from before capture
            // attached pops with no recorded open.
            debug!(
                "orphan nvtx range pop on thread {thread_id} in domain 0x{domain:X} \
                 with no open push; skipping"
            );
            return None;
        };

        // Retain only keys with open ranges, so the map does not grow with
        // every thread the process ever ran.
        if now_empty {
            self.stacks.remove(&key);
        }
        Some(open.close(end, parent, false))
    }

    /// Close every push still open at the end of the trace.
    ///
    /// Nesting is the stack's own shape, so a leaked inner push keeps its
    /// parent rather than being flattened. Stacks are visited in key order for
    /// determinism.
    pub(crate) fn close_at_trace_end(
        &mut self,
        trace_end: TimeUnixNanoSec,
    ) -> Vec<(SpanId, NvtxSpan)> {
        let mut leaked: Vec<(StackKey, Vec<OpenPushSpan>)> = self.stacks.drain().collect();
        leaked.sort_by_key(|(key, _)| *key);

        let unpopped: usize = leaked.iter().map(|(_, stack)| stack.len()).sum();
        if unpopped > 0 {
            warn!(
                "{unpopped} nvtx range(s) were never popped; closing them at trace end ({trace_end})"
            );
        }

        let mut closed = Vec::new();
        for ((thread_id, domain), mut stack) in leaked {
            // Drained innermost-first so the parent is whatever the pop
            // uncovers — the same rule `pop` applies, not a second statement of
            // it that has to be kept in step.
            while let Some(open) = stack.pop() {
                let parent = stack.last().map(|enclosing| enclosing.id);
                debug!(
                    "nvtx range \"{}\" on thread {thread_id} in domain 0x{domain:X} \
                     was never popped",
                    open.range.name
                );
                closed.push(open.close(trace_end, parent, true));
            }
        }
        closed
    }
}

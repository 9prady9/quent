// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Resource lifespan reconstruction — the third match key.
//!
//! The interval between `nvtxDomainResourceCreate` and its destroy, modelled as
//! an [`NvtxSpan`] with [`SpanKind::Resource`].
//!
//! **The match key is the handle alone**, not `(domain, handle)` — forced by the
//! vocabulary, since
//! [`NvtxEvent::ResourceDestroy`](nvtx_events::NvtxEvent::ResourceDestroy)
//! carries only a handle because the underlying NVTX call does. Keying on the
//! pair would not fail loudly: every destroy would miss its create and every
//! resource would silently reconstruct as a leak closed at trace end. The domain
//! is recovered from the create instead.
//!
//! A create says a handle exists and what it is called — nothing about its size
//! or occupancy — so nothing further is inferred.

use std::collections::HashMap;

use quent_time::TimeUnixNanoSec;
use tracing::{debug, warn};

use crate::span::{NvtxSpan, SpanKind};

/// Label an `nvtxResourceAttributes_t::identifierType` tag.
///
/// `nvToolsExt.h` composes an identifier type as
/// `NVTX_RESOURCE_MAKE_TYPE(CLASS, INDEX) = (CLASS << 16) | INDEX`, and
/// `NVTX_RESOURCE_CLASS_GENERIC` is `1` — so the core `nvtxResourceGenericType_t`
/// set is `0x0001_0001`..=`0x0001_0004`. Other classes (CUDA, CUDA runtime,
/// OpenCL, D3D, sync) are extension surfaces this crate deliberately does not
/// interpret.
///
/// Total by construction: everything outside the core set passes through as
/// `"<identifier_type {n}>"`, interpolating only the raw integer, so an
/// unrecognized type can never render as a recognized one.
pub(crate) fn label_identifier_type(identifier_type: i32) -> String {
    // Confirmed against the pixi-pinned nvtx-c headers, the same ones
    // `nvtx-injection`'s bindgen build reads.
    let label = match identifier_type {
        // `NVTX_RESOURCE_TYPE_UNKNOWN`, also what the capture layer records for
        // an attribute struct too short to contain the field — a legitimate
        // "not stated" rather than an anomaly.
        0x0000_0000 => "unknown",
        0x0001_0001 => "generic pointer",
        0x0001_0002 => "generic handle",
        0x0001_0003 => "native thread",
        0x0001_0004 => "posix thread",
        _ => return format!("<identifier_type {identifier_type}>"),
    };
    label.to_owned()
}

/// A `ResourceCreate` awaiting its matching `ResourceDestroy`.
struct OpenResource {
    /// Recovered from the create, because the destroy carries no domain.
    domain: u64,
    name: String,
    identifier_type: i32,
    start: TimeUnixNanoSec,
}

impl OpenResource {
    /// Close this resource, clamping `end` up to `start` as the range
    /// reconstructions do.
    fn close(self, end: TimeUnixNanoSec, synthetic_end: bool) -> NvtxSpan {
        NvtxSpan {
            domain: self.domain,
            // A resource is not owned by the thread that announced it.
            thread_id: None,
            name: self.name,
            // `nvtxResourceAttributes_t` is not an event-attribute struct; it
            // has no category, color, or payload to carry.
            category: None,
            color: None,
            payload: None,
            start: self.start,
            end: end.max(self.start),
            kind: SpanKind::Resource,
            identifier_type_label: Some(label_identifier_type(self.identifier_type)),
            // Resource lifespans do not nest.
            parent: None,
            synthetic_end,
        }
    }
}

/// The set of currently-open resource lifespans, keyed by handle alone.
#[derive(Default)]
pub(crate) struct Resources {
    open: HashMap<u64, OpenResource>,
}

impl Resources {
    /// Record a `ResourceCreate` under its already-resolved `name`.
    ///
    /// Returns a span when this create *displaces* a lifespan still open under
    /// the same handle. The displaced lifespan is closed at the recreate and
    /// flagged synthetic rather than discarded — the create was observed.
    pub(crate) fn create(
        &mut self,
        handle: u64,
        domain: u64,
        name: String,
        identifier_type: i32,
        start: TimeUnixNanoSec,
    ) -> Option<NvtxSpan> {
        let open = OpenResource {
            domain,
            name,
            identifier_type,
            start,
        };
        let displaced = self.open.insert(handle, open)?;
        warn!(
            "nvtx resource handle 0x{handle:X} was recreated before it was destroyed; closing \
             the earlier lifespan at the recreate ({start})"
        );
        Some(displaced.close(start, true))
    }

    /// Close the resource matching `handle`, if one is open.
    ///
    /// Returns `None` for an orphan destroy — the *normal* case for a resource
    /// created before capture attached.
    pub(crate) fn destroy(&mut self, handle: u64, end: TimeUnixNanoSec) -> Option<NvtxSpan> {
        let Some(open) = self.open.remove(&handle) else {
            // `debug!`, not `warn!`, because this is routine: logging it as an
            // anomaly would bury the genuine ones.
            debug!(
                "orphan nvtx resource destroy for handle 0x{handle:X} with no open create; skipping"
            );
            return None;
        };
        Some(open.close(end, false))
    }

    /// Close every resource still open at the end of the trace.
    ///
    /// Ordered by `(start, handle)` so several leaked resources still
    /// reconstruct deterministically out of an unordered `HashMap`.
    pub(crate) fn close_at_trace_end(&mut self, trace_end: TimeUnixNanoSec) -> Vec<NvtxSpan> {
        let mut leaked: Vec<(u64, OpenResource)> = self.open.drain().collect();
        leaked.sort_by_key(|(handle, open)| (open.start, *handle));

        // One `warn!` for the count, per-handle detail at `debug!`, so a stream
        // leaking many resources does not emit one warning each.
        if !leaked.is_empty() {
            warn!(
                "{} nvtx resource(s) were never destroyed; closing them at trace end ({trace_end})",
                leaked.len()
            );
        }

        leaked
            .into_iter()
            .map(|(handle, open)| {
                debug!("nvtx resource handle 0x{handle:X} was never destroyed");
                open.close(trace_end, true)
            })
            .collect()
    }
}

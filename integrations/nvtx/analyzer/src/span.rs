// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! The reconstruction core's own plain span types.
//!
//! Hand-written structs, not framework types. Keeping our own is what lets the
//! core stay tolerant: zero-duration and synthetically-closed spans are
//! representable here, where the shared framework would reject or panic on
//! them.

use nvtx_events::{NvtxColor, NvtxPayload};
use quent_time::TimeUnixNanoSec;

/// A stable handle to an [`NvtxSpan`] within one reconstructed model.
///
/// The index into the owning model's span list. Only meaningful against the
/// model it was produced from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanId(pub usize);

/// Which NVTX construct a span was reconstructed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// A per-thread nested range: `nvtxDomainRangePushEx` / `nvtxDomainRangePop`.
    PushPop,
    /// A process-wide range keyed by id: `nvtxDomainRangeStartEx` / `nvtxDomainRangeEnd`.
    StartEnd,
    /// A resource lifespan: `nvtxDomainResourceCreate` / `nvtxDomainResourceDestroy`.
    Resource,
}

/// Map a raw category id onto the presence it encodes.
///
/// Category `0` is NVTX's "no category" sentinel — an absence, not an id. Every
/// consumer goes through here so the rule is stated once. `nvtx-events` would
/// be the better home, since it already models `color`, `message`, and
/// `payload` as `Option` and `category` is the one attribute left as a magic
/// number.
pub(crate) const fn category_id(raw: u32) -> Option<u32> {
    if raw == 0 { None } else { Some(raw) }
}

/// A reconstructed NVTX interval.
///
/// `start <= end` is an invariant established at construction — out-of-order
/// pairs are clamped rather than rejected, so [`Self::duration`] can never
/// underflow.
#[derive(Debug, Clone, PartialEq)]
pub struct NvtxSpan {
    /// Raw domain handle (`0` = default domain).
    pub domain: u64,
    /// Raw OS thread id, when the originating events carried one.
    pub thread_id: Option<u32>,
    /// Resolved message, or a placeholder when the handle was never registered.
    pub name: String,
    /// Raw category id (`None` when the event carried category `0` = none).
    pub category: Option<u32>,
    /// Verbatim color attribute, undecoded.
    pub color: Option<NvtxColor>,
    /// Verbatim payload value, undecoded.
    pub payload: Option<NvtxPayload>,
    pub start: TimeUnixNanoSec,
    /// Always `>= start`.
    pub end: TimeUnixNanoSec,
    /// Which NVTX construct this span was reconstructed from.
    pub kind: SpanKind,
    /// What kind of thing a [`SpanKind::Resource`] span identifies.
    ///
    /// `None` for every other kind. A core/generic NVTX resource type gets a
    /// static label; an unrecognized or CUDA-extension type passes through as
    /// `"<identifier_type {n}>"` rather than being guessed at.
    pub identifier_type_label: Option<String>,
    /// The enclosing span, for nested push/pop ranges.
    pub parent: Option<SpanId>,
    /// `true` when the close was never observed and was synthesized at trace end.
    pub synthetic_end: bool,
}

impl NvtxSpan {
    /// The span's duration in nanoseconds.
    ///
    /// Saturating, so a malformed stream can never underflow it.
    pub fn duration(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

/// A reconstructed `nvtxDomainMarkEx` instant.
///
/// Deliberately *not* a zero-length [`NvtxSpan`]: a consumer rendering a
/// timeline needs to tell "happened at" from "lasted zero".
#[derive(Debug, Clone, PartialEq)]
pub struct NvtxMark {
    /// Raw domain handle (`0` = default domain).
    pub domain: u64,
    /// Raw OS thread id, when the originating event carried one.
    pub thread_id: Option<u32>,
    /// Resolved message, or a placeholder when the handle was never registered.
    pub name: String,
    /// Raw category id (`None` when the event carried category `0` = none).
    pub category: Option<u32>,
    /// Verbatim color attribute, undecoded.
    pub color: Option<NvtxColor>,
    /// Verbatim payload value, undecoded.
    pub payload: Option<NvtxPayload>,
    /// When the mark was emitted.
    pub timestamp: TimeUnixNanoSec,
}

/// A domain the stream referenced, with its resolved name and lifespan.
///
/// Present for every domain *mentioned* by the stream, not only those with a
/// captured `nvtxDomainCreate` — a domain created before capture began still
/// groups the events that name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvtxDomain {
    /// Raw domain handle (`0` = default domain).
    pub domain: u64,
    /// Resolved name, or the placeholder for an uncreated handle.
    pub name: String,
    /// The `nvtxDomainCreate` timestamp, or the first time the domain was seen.
    pub created: TimeUnixNanoSec,
    /// The `nvtxDomainDestroy` timestamp, if one was captured.
    pub destroyed: Option<TimeUnixNanoSec>,
}

/// An OS thread the stream referenced, with its resolved name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvtxThread {
    /// Raw OS thread id (the `nvtxNameOsThread` id space).
    pub thread_id: u32,
    /// Resolved name, or `"thread {id}"` when the thread was never named.
    pub name: String,
}

/// A category the stream referenced, namespaced by its owning domain.
///
/// The `(domain, category)` pair *is* the identity: NVTX category ids are only
/// unique within a domain, so a globally-keyed view would silently merge
/// unrelated categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvtxCategory {
    /// Raw domain handle the category belongs to.
    pub domain: u64,
    /// Raw category id (never `0` — that is the "no category" sentinel).
    pub category: u32,
    /// Resolved name, or the placeholder for an unnamed category.
    pub name: String,
}

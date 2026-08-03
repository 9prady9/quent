// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Range statistics — a fold over the reconstructed span set.
//!
//! Groups range spans by `(name, domain, category)`. All three are needed: the
//! same name means different work in different domains, and the application
//! deliberately distinguishes categories.
//!
//! Only ranges participate. Marks are instants with no duration to average, and
//! resource lifespans measure how long a handle existed rather than how long
//! work took — folding either in would produce a number that reads like a
//! duration but answers a different question.
//!
//! Synthetically-closed spans contribute to every figure *and* to
//! [`RangeStats::synthetic_count`]. Their end is inferred, so their duration is
//! a lower bound: dropping them would understate the count, folding them in
//! silently would overstate the confidence.

use std::collections::{BTreeMap, HashMap};

use crate::span::{NvtxSpan, SpanKind};

/// What one [`RangeStats`] group is keyed by.
///
/// `Ord` rather than `Hash`: the statistics come back in a [`BTreeMap`], so
/// repeated builds of the same stream iterate in the same order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatsKey {
    /// The resolved span name (or its placeholder).
    pub name: String,
    /// Raw domain handle (`0` = default domain).
    pub domain: u64,
    /// Raw category id, or `None` for NVTX's "no category" sentinel.
    pub category: Option<u32>,
}

/// Aggregated durations for one `(name, domain, category)` group, in nanoseconds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RangeStats {
    /// How many spans fell into this group.
    pub count: u64,
    /// Sum of every span's duration.
    pub total_duration: u64,
    /// `total_duration / count`, or `0` for an empty group.
    pub avg_duration: u64,
    /// The shortest span's duration.
    pub min_duration: u64,
    /// The longest span's duration.
    pub max_duration: u64,
    /// How many of [`Self::count`] were closed at trace end rather than observed.
    ///
    /// Their durations are lower bounds, so this is what tells a consumer how
    /// much of the group is inferred rather than measured.
    pub synthetic_count: u64,
}

impl RangeStats {
    /// Fold one span into this group.
    fn accumulate(&mut self, span: &NvtxSpan) {
        let duration = span.duration();

        if self.count == 0 {
            // A zeroed `min` would win every comparison, so the first span seeds
            // both bounds rather than being compared against them.
            self.min_duration = duration;
            self.max_duration = duration;
        } else {
            self.min_duration = self.min_duration.min(duration);
            self.max_duration = self.max_duration.max(duration);
        }

        self.count += 1;
        // Saturating: a saturated total is a visibly wrong number rather than a
        // silently wrapped plausible one.
        self.total_duration = self.total_duration.saturating_add(duration);
        if span.synthetic_end {
            self.synthetic_count += 1;
        }
    }

    /// Compute the derived average, once the group is complete.
    ///
    /// `checked_div` because an empty group is representable, and dividing by
    /// zero would be the one arithmetic panic in an otherwise total path.
    fn finish(&mut self) {
        self.avg_duration = self.total_duration.checked_div(self.count).unwrap_or(0);
    }
}

/// Aggregate every range span in `spans` by `(name, domain, category)`.
pub(crate) fn range_statistics(spans: &[NvtxSpan]) -> BTreeMap<StatsKey, RangeStats> {
    // Folded against borrowed names, then keyed by owned ones once per group.
    // Grouping straight into the `BTreeMap` would clone every span's name only
    // to drop it again on the already-present path.
    let mut grouped: HashMap<(&str, u64, Option<u32>), RangeStats> = HashMap::new();

    for span in spans
        .iter()
        .filter(|span| matches!(span.kind, SpanKind::PushPop | SpanKind::StartEnd))
    {
        grouped
            .entry((span.name.as_str(), span.domain, span.category))
            .or_default()
            .accumulate(span);
    }

    grouped
        .into_iter()
        .map(|((name, domain, category), mut stats)| {
            // `avg` is derived, so it is computed once per group rather than
            // recomputed on every span.
            stats.finish();
            let key = StatsKey {
                name: name.to_owned(),
                domain,
                category,
            };
            (key, stats)
        })
        .collect()
}

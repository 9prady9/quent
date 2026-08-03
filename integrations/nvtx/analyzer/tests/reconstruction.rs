// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Reconstruction of `RangeStart`/`RangeEnd` pairs, and the tolerance guarantees
//! that hold while doing it.

mod fixtures;

use fixtures::{mark, range_end, range_start, span};
use nvtx_analyzer::{NvtxModelBuilder, SpanKind};

#[test]
fn startend_match_by_handle() {
    // The final `RangeEnd` carries a *different* domain than its start: the
    // match key is `range_id` alone, which NVTX makes process-globally unique.
    let events = vec![
        range_start(100, 1, 7, "alpha"),
        range_start(150, 2, 9, "beta"),
        range_end(200, 1, 7),
        range_end(250, 999, 9),
    ];

    let model = NvtxModelBuilder::build(events);

    assert_eq!(model.spans().len(), 2, "one span per matched range");

    let alpha = span(&model, "alpha");
    assert_eq!(alpha.kind, SpanKind::StartEnd);
    assert_eq!(alpha.domain, 1);
    assert_eq!(alpha.start, 100);
    assert_eq!(alpha.end, 200);
    assert_eq!(alpha.duration(), 100);
    assert!(!alpha.synthetic_end, "the end was observed");

    let beta = span(&model, "beta");
    assert_eq!(beta.kind, SpanKind::StartEnd);
    assert_eq!(beta.domain, 2, "domain comes from the start, not the end");
    assert_eq!(beta.start, 150);
    assert_eq!(beta.end, 250);
    assert!(!beta.synthetic_end);
}

#[test]
fn out_of_order_sorted() {
    // The end arrives before its start.
    let events = vec![range_end(200, 1, 7), range_start(100, 1, 7, "late")];

    let model = NvtxModelBuilder::build(events);

    assert_eq!(model.spans().len(), 1);
    let late = span(&model, "late");
    assert!(late.start <= late.end, "replay is timestamp-ordered");
    assert_eq!(late.start, 100);
    assert_eq!(late.end, 200);
    assert!(!late.synthetic_end, "the end was observed, just early");
}

#[test]
fn duplicate_timestamps_no_panic() {
    let stream = || {
        vec![
            range_start(100, 1, 7, "a"),
            range_start(100, 1, 8, "b"),
            range_end(100, 1, 7),
            range_end(100, 1, 8),
        ]
    };

    let model = NvtxModelBuilder::build(stream());
    assert_eq!(model.spans().len(), 2);

    for span in model.spans() {
        assert_eq!(span.start, 100);
        assert_eq!(span.end, 100, "zero-duration spans are legal");
        assert_eq!(span.duration(), 0);
    }

    // Equal timestamps preserve arrival order, so a rebuild is identical.
    let again = NvtxModelBuilder::build(stream());
    assert_eq!(
        model.spans(),
        again.spans(),
        "duplicate timestamps reconstruct deterministically"
    );
}

#[test]
fn unclosed_start_closed_synthetic() {
    // `leaked` is never ended; the mark sets the trace end.
    let events = vec![range_start(100, 1, 7, "leaked"), mark(500, 1, "tick")];

    let model = NvtxModelBuilder::build(events);

    assert_eq!(model.spans().len(), 1, "a mark is an instant, not a span");
    let leaked = span(&model, "leaked");
    assert_eq!(leaked.start, 100);
    assert_eq!(leaked.end, 500, "closed at the max observed timestamp");
    assert!(leaked.synthetic_end, "the end was never observed");
}

#[test]
fn restarted_id_keeps_the_displaced_range() {
    // `first` is restarted under the same id before it ever ended. NVTX ids are
    // process-globally unique, so this is malformed — but the first start was
    // observed, and dropping it would erase a range the stream really reported.
    let events = vec![
        range_start(100, 1, 7, "first"),
        range_start(300, 1, 7, "second"),
        range_end(500, 1, 7),
    ];

    let model = NvtxModelBuilder::build(events);

    assert_eq!(model.spans().len(), 2, "both starts produce a span");

    let first = span(&model, "first");
    assert_eq!(first.start, 100);
    assert_eq!(
        first.end, 300,
        "the displaced range is closed at the restart, not dropped"
    );
    assert!(
        first.synthetic_end,
        "its close was never observed, so it is flagged synthetic"
    );

    // The `RangeEnd` belongs to whichever start was open when it arrived.
    let second = span(&model, "second");
    assert_eq!(second.start, 300);
    assert_eq!(second.end, 500);
    assert!(!second.synthetic_end, "the end was observed");
}

#[test]
fn orphan_end_skipped() {
    // A `RangeEnd` with no open start is logged and skipped, not fatal.
    let events = vec![
        range_end(100, 1, 7),
        range_start(200, 1, 8, "real"),
        range_end(300, 1, 8),
    ];

    let model = NvtxModelBuilder::build(events);

    assert_eq!(model.spans().len(), 1, "the orphan end produced no span");
    let real = span(&model, "real");
    assert_eq!(real.start, 200);
    assert_eq!(real.end, 300);
}

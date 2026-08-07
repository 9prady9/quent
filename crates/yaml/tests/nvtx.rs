// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Model-level NVTX configuration tests.

use quent_yaml::parse_from_str;

const MODEL: &str = "\
quent: alpha
model: m
entities:
  Engine:
    events:
      started: {}
";

#[test]
fn nvtx_defaults_to_enabled() {
    assert!(parse_from_str(MODEL, None).expect("parses").nvtx);
}

#[test]
fn nvtx_can_be_disabled() {
    let model = MODEL.replace("model: m", "model: m\nnvtx: false");
    assert!(!parse_from_str(model, None).expect("parses").nvtx);
}

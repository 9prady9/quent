// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    quent_codegen_tutorial_build::build_python(
        "finite-state-machine",
        "quent_tutorial_finite_state_machine",
    )
}

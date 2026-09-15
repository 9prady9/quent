// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    quent_codegen_tutorial_build::build_python(
        "untyped-entity-references",
        "quent_tutorial_untyped_entity_references",
    )
}

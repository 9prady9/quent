// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = manifest_dir.join("../../../../../examples/readme/model.yaml");
    println!("cargo:rerun-if-changed={}", model.display());
    let schema = quent_yaml::parse_from_file(model)?.schema;
    let options = quent_schema_codegen_cpp::Options {
        crate_name: "quent-demo-cpp-test".to_owned(),
        instrumentation_path: "quent_readme_example".to_owned(),
        exporters: quent_schema_codegen_cpp::Exporters {
            ndjson: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let files = quent_schema_codegen_cpp::emit(&schema, &options)?;
    let bridges = quent_schema_codegen_cpp::write_bridge_files(&files, &options)?;
    let example = manifest_dir.join("../example/readme/main.cpp");
    let test = manifest_dir.join("test.cpp");
    println!("cargo:rerun-if-changed={}", example.display());
    println!("cargo:rerun-if-changed={}", test.display());
    let mut build = cxx_build::bridges(bridges);
    let include_dir = quent_schema_codegen_cpp::stage_cxx_headers(&options)?;
    build
        .include(&include_dir)
        .file(example)
        .file(test)
        .define("QUENT_DEMO_LIBRARY", None)
        .define(
            "QUENT_CPP_BRIDGE_HEADER",
            Some("\"quent-demo-cpp-test/gen/quent.hpp\""),
        )
        .std("c++20")
        .compile("quent_demo_cpp_test");
    Ok(())
}

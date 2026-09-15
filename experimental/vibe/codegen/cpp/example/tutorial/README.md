# C++ schema tutorials

Each directory mirrors the tutorial with the same name in
`crates/yaml/examples`. Its bridge build reads that tutorial's canonical
`model.yaml` and compiles `main.cpp` against the generated C++ façade.

Build one tutorial from the repository root:

```sh
pixi run cargo build --manifest-path \
  experimental/vibe/codegen/cpp/example/tutorial/minimal-model/bridge/Cargo.toml
```

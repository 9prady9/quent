# Python schema tutorials

Each directory mirrors the tutorial with the same name in
`crates/yaml/examples`. Its bridge build reads that tutorial's canonical
`model.yaml` and packages generated PEP 561 stubs with the extension module.

Build and run one tutorial from the repository root:

```sh
pixi run maturin develop --uv -m \
  experimental/vibe/codegen/python/example/tutorial/minimal-model/bridge/Cargo.toml
pixi run python \
  experimental/vibe/codegen/python/example/tutorial/minimal-model/main.py
```

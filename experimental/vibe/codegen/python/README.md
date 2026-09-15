# Schema PyO3 generator

`quent-schema-codegen-python` emits a PyO3 module and matching PEP 561 type
stubs. `Context` exposes scoped observers that construct UUID-owning handles.
Event fields are keyword-only and events carrying entity-reference data accept
typed mappings. UUID values use the standard-library `uuid.UUID` type.

```rust
let schema = quent_yaml::parse_from_file("model.yaml")?.schema;
let options = quent_schema_codegen_python::Options {
    module_name: "application_events".to_owned(),
    instrumentation_path: "application_instrumentation::model".to_owned(),
    exporters: quent_schema_codegen_python::Exporters::all(),
    ..Default::default()
};
quent_schema_codegen_python::write_generated_files(
    &quent_schema_codegen_python::emit(&schema, &options)?,
    std::env::var("OUT_DIR")?,
)?;
```

Records are accepted as mappings. Targeted entity references accept either a
standard-library `uuid.UUID` or the matching generated handle; references
carrying data accept a mapping with `target` and `data` fields.

Dynamic-attribute fields accept mappings. `None`, `bool`, `int`, `float`,
`str`, and nested mappings have natural conversions; `DynamicValue` selects an
exact numeric width or constructs a typed list, structure, structure list, or
recursive list with `DynamicValue.list`.
Mapping iteration order is retained for attributes and nested structure
members; sequence order is retained for lists.

FSM transitions consume the source-state handle and return a generated handle
for the target state. Type checkers and editor completion therefore expose only
legal transitions. Reusing a consumed handle raises `HandleConsumedError`;
transition sequence numbers are assigned by the instrumentation runtime and are
not Python parameters.

`Context()` creates a no-op context. `Context.close()` prevents creation of new
observers; existing observers and handles retain their scoped telemetry runtime.
Exporter shutdown waits until the last observer or handle is released.
Once-cardinality events on non-FSM entities expose `<event>_emitted()`
predicates. Generated exceptions derive from `QuentError`. Nested options are
rejected during generation because Python cannot distinguish `None` from
`Some(None)`.

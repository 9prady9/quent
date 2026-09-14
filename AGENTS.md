# Agent instructions

## Temporary schema migration freeze

Major refactors are in progress to remove proof-of-concept technical debt and
complete the schema-based instrumentation architecture:

- [#638: schema-based C++ and Python generators][pr-638]
- [#699: schema-based instrumentation architecture][pr-699]

Before starting work that affects the areas below, check whether both pull
requests have merged. Until both have merged, do not make significant changes
to:

- `quent-model` and `quent-model-macros` (`crates/model`,
  `crates/model-macros`)
- `quent-codegen` (`crates/codegen`)
- `quent-stdlib` and `quent-query-engine-model` (`crates/stdlib`,
  `domains/query_engine/model`)
- related examples, tests, and minor crates being removed or migrated by either
  pull request

If requested work overlaps these areas while either pull request remains open,
report the conflict and ask the user how to proceed before editing. This
restriction expires after both pull requests merge.

[pr-638]: https://github.com/rapidsai/quent/pull/638
[pr-699]: https://github.com/rapidsai/quent/pull/699

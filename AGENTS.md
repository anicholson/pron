# pron — Agent Guide

## Contract Files

- **`TEST_TREES.md`** — the project's test trees (the behaviour contract). Every expected behaviour and side effect lives here as EARS-pattern trees.
- **`MENTAL_MODEL.md`** — the project's mental model (domain identity, ubiquitous language, invariants, decisions).
- **`.opencode/contree.md`** — the contree methodology rules and development contract.

## Testing

### Test framework

- **Runner**: cargo-nextest (process-per-test isolation, parallel, better failure output than `cargo test`).
- **Tree output**: `make test` pipes nextest's libtest-JSON through `xtask`, which re-nests `::`-separated module paths into indented tree output with pass/fail icons.
- **Mutation**: cargo-mutants with nextest, scoped to `--lib` (Domain + Use-case only).
- **CI**: `make test-ci` runs nextest with the `ci` profile (retries + JUnit XML at `target/nextest/ci/junit.xml`).

### Layer commands

| Layer | Command | Notes |
|---|---|---|
| All (tree) | `make test` | All tests, tree-formatted |
| Unit (tree) | `make test-lib` | Domain + Use-case + driving Adapter (`--lib`) |
| Integration (tree) | `make test-integration` | Driven Adapter + System + Journey (`tests/`) |
| Flat output | `make test-flat` | Raw nextest, no tree formatter |
| Watch | `make test-watch` | watchexec re-runs unit tests on `.rs` changes |
| Mutation | `make test-mutate` | cargo-mutants, `pron` crate, `--lib` only |
| CI | `make test-ci` | Retries + JUnit XML |

### Per-layer filtersets

Domain and Use-case tests are both inline `#[cfg(test)] mod tests` (Rust convention). Separate them by module path:

```sh
cargo nextest run --lib -E 'test(/pron::domain/)'        # Domain only
cargo nextest run --lib -E 'test(/pron::application/)'    # Use-case only
```

Integration tests live in `tests/` as separate binaries. Run a specific one:

```sh
cargo nextest run --test <binary_name>     # e.g. --test system_pron
```

### Test structure convention

Module nesting is the describe/it hierarchy. Rust has no `describe`/`it`; the idiomatic mirror is:

```rust
#[cfg(test)]
mod tests {
    mod when_a_minute_matches {
        #[test]
        fn then_the_job_fires() { /* ... */ }
    }
}
```

The test path `pron::scheduler::tests::when_a_minute_matches::then_the_job_fires` renders in the tree formatter as:

```
pron
  scheduler
    tests
      when_a_minute_matches
        ✓ then_the_job_fires
```

Sync compares tree text against module/test structure after slug normalisation (spaces → underscores, EARS keyword preserved as first token).

### In-memory twins and contract suites

In-memory twins live in `src/application/ports/*.rs` (under `in_memory` submodules), gated `#[cfg(any(test, feature = "test-support"))]` and `pub`. The `test-support` feature is enabled in `[dev-dependencies]` so integration tests can use them.

Shared port contract suites are `macro_rules!` that expand to nested `mod`/`#[test]`, invoked in both the in-memory unit test module and the real-adapter integration test file.

### Outside-in TDD workflow

1. Write one failing Journey test (spawns the real binary, drives the full arc).
2. Descend through layers: System → driving adapter → Use-case → Domain → Port contract → driven adapter. Write one failing test at each layer.
3. Implement at the lowest layer, fold back up.
4. Run mutation testing at the end (`make test-mutate`).

## Architectural rules

Hexagonal: domain pure, I/O in adapters, dependencies point inward.

- `src/domain/` must not import from `src/application/` or `src/adapters/`.
- `src/application/` must not import concrete adapters — only port traits from `src/application/ports/`.
- No circular dependencies.

Enforcement: a `tests/arch.rs` test that walks `use`/`mod` statements and asserts direction. cargo-modules (`cargo modules tree`) visualises the structure but does not enforce rules. The arch test is the enforcement mechanism; create it when the first domain module lands.

## Linting

```sh
make lint       # cargo clippy -- -D warnings
```

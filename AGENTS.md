# AGENTS.md — Shelly GPUI Project

## Engineering Standards: Zero Deadcode Policy & No Warning Suppression

- **NEVER ALLOW DEADCODE**: Never use `#[allow(dead_code)]`, `#[allow(unused...)]`, or language equivalents to silence compiler warnings.
- **NEVER CONCEAL DATA**: Compiler warnings are diagnostic telemetry. Concealing warnings or dead code from the operator is strictly forbidden.
- **WIRING OR STRICT YAGNI**: Every struct, enum variant, field, function, method, and event must either be actively wired to real consumers and callers in the current slice, or deleted immediately (strict YAGNI). Do not keep speculative stubs for future slices.
- **HARD COMPILER ENFORCEMENT**: All Rust crates in this repository must enforce `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, and `#![deny(unused_must_use)]` at their crate root.

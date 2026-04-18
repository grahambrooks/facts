# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`facts` is a Midje-inspired Rust test library. Assertions read left-to-right as `actual => expected` via the `fact!` / `refute!` macros, where the RHS is any value that implements the `Checker` trait. See `design.md` for the full proposal, including features not yet built (`facts!` group macro, `tabular!`, `provided!` for mocking, `background!`).

## Commands

- `cargo build` — compile the library.
- `cargo test` — run unit, integration, and doc tests.
- `cargo test --test smoke literal_equality` — run a single integration test by name.
- `cargo test --doc` — run just the doctest in `src/lib.rs`.

## Architecture

The design hinges on one non-obvious trick: **the blanket `impl<T: PartialEq + Debug> Checker<T> for T` coexists with per-checker impls without a specialization conflict, because custom checkers are distinct types.** Reading both files together is necessary to see why:

- `src/lib.rs` defines `trait Checker<T>` and the blanket impl, plus the `fact!` / `refute!` macros. Both macros bind `expected` and `actual` to hygienic locals, then dispatch via `Checker::check(&expected, &actual)`.
- `src/checkers.rs` defines checker structs (`Roughly`, `Contains<T>`, `IsOk`, …) and impls `Checker<Target>` for each. Because `Roughly` is not `f64`, there's no overlap with the blanket impl — `Checker<f64>` is provided by `f64` (via blanket) and separately by `Roughly` (via explicit impl). The compiler picks based on the RHS type, so `fact!(x => 4)` and `fact!(x => roughly(4.0, 0.1))` both work with a single macro arm.

Consequences when extending:

- A new checker is a struct + constructor fn + one or more `impl Checker<TargetType> for MyChecker` blocks. No macro changes needed.
- Do not give checker types a `PartialEq` impl against their target type — that would collide with the blanket.
- Checker constructors are plain fns (`roughly(…)`, `is_ok()`), not constants, to match Midje's call-site style and keep zero-sized types simple.

## Design constraints worth preserving

- **Negation is `refute!`, not `=not=>`.** The token `!=>` tangles with `:expr` capture (prefix `!`), and a custom three-token arrow complicates macro parsing. A separate macro is clearer.
- **Every fact stays a plain `#[test]`-callable assertion.** No global state, no custom harness — `cargo test` filters, parallelism, and `--nocapture` keep working.
- **Mocking (future `provided!`) will target trait seams only.** Rust has no `with-redefs`; the library will steer users to trait objects rather than fake it.

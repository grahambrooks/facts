# `facts` — a Midje-inspired Rust test library

## Core idea

Midje's power comes from the `=>` arrow: a single reading order — *actual → expected* — that scales from literals to rich checkers to mocks. Rust can keep that feel because `=>` is a valid tt separator in declarative macros.

## Surface syntax

```rust
use facts::prelude::*;

facts! { "arithmetic"
    fact!(2 + 2 => 4);
    fact!(sqrt(2.0) => roughly(1.414, 0.001));
    fact!(parse("hi") => is_ok());
    fact!(vec![1, 2, 3] => contains(2));
    fact!(name() => has_prefix("Dr."));
    refute!(bad_call() => 0);
}
```

`facts!` expands to one `#[test] fn` per named block; `fact!` expands to an assertion that prints actual, expected, and the description on failure. Negation is a separate macro `refute!` rather than a fused `=not=>` arrow — `!=>` would tangle with Rust's prefix `!` in `:expr` capture.

## Checker unification

Everything on the RHS of `=>` is a `Checker`. A blanket impl makes literals work without a specialization conflict: custom checkers are distinct types (`Roughly`, `Contains<T>`, …) that impl `Checker<Target>` for the type being checked, while the blanket `impl<T: PartialEq + Debug> Checker<T> for T` handles equality.

```rust
pub trait Checker<T: ?Sized> {
    fn check(&self, actual: &T) -> Outcome;
}

impl<T: PartialEq + Debug> Checker<T> for T { /* eq check */ }
```

So `=> 4` and `=> roughly(1.414, 0.001)` go through the same path. Built-ins planned: `eq`, `roughly`, `truthy`, `falsey`, `contains`, `has_prefix`/`suffix`, `matches(regex)`, `is_ok`/`is_err`/`is_some`/`is_none`, `any_of`/`all_of`/`not`.

## Tabular facts

Midje's tabular form with the body template first, then the header tuple, then rows:

```rust
tabular! {
    fact!(a + b => sum),
    (a, b, sum),
    (1, 2, 3),
    (4, 5, 9),
    (-1, 1, 0),
}
```

The original `|a, b, sum|` row delimiters from the first sketch conflict with Rust's closure grammar in `:expr` capture, so the shipped form uses parenthesized tuples throughout. Each row expands to `{ let (a, b, sum) = (1, 2, 3,); fact!(a + b => sum); }`. Homogeneous column types are required (rust type-checks the generated `let` bindings). Single-column tables need a trailing comma in the header: `(n,)`.

## Mocking via `provided`

Midje's `provided` redefines functions inside one fact. Rust can't do that dynamically, so the library only supports it for **trait objects passed as dependencies** — the idiomatic mock point. Free functions are out of scope — the library steers users to trait-based seams rather than pretending Rust has `with-redefs`.

Two macros cooperate:

```rust
mockable! {
    pub trait Clock as ClockMock {
        fn now(&self) -> u32;
        fn tzname(&self) -> String;
    }
}

#[test]
fn greets_in_the_morning() {
    let clock = ClockMock::new();
    provided! {
        clock.now() => 8;
        clock.tzname() => "UTC".to_string();
    }
    fact!(greet(&clock) => "Good morning");
}
```

`mockable!` defines both the trait and a sibling mock struct. The mock exposes each method as a `pub` field of type `RefCell<Option<Return>>`; the trait impl panics on an unset slot. `provided!` is pure sugar for `*mock.method.borrow_mut() = Some(value);`. Return types must be `Clone` (slots are cloned on each call).

The `as ClockMock` form avoids needing a `paste`-style ident-synthesis dependency — users explicitly name the mock. Deferred to a later iteration: call-count verification, argument matchers, `&mut self` methods, generic methods.

## Backgrounds

```rust
facts! { "db queries"
    background!(let db = TestDb::new(); before_each);
    fact!(db.count() => 0);
    fact!({ db.insert("x"); db.count() } => 1);
}
```

`background!` lowers to a helper closure that wraps each generated test function.

## Failure output

The aim is Midje-like: show the expression *source text* (captured by the macro via `stringify!`), the actual value, the checker's description, and the fact's label:

```
FAIL about addition: "adds positives"
  expected: sqrt(2.0) => roughly(1.414, 0.001)
  actual:   1.4142135
  reason:   |1.4142135 - 1.414| = 0.0002135 exceeds 0.001
```

## What we'd *not* port

- **Arrow family overload** (`=future=>`, `=streams=>`, etc.) — cute in Clojure, noisy in Rust. Single `=>` plus `refute!` is enough.
- **Autotest watcher** — `cargo-watch` already exists.
- **`midje.sweet` global state** — every fact stays a pure `#[test]`, so `cargo test --no-capture`, filters, and parallelism keep working.

## Crate layout (target)

```
facts/            # declarative macros + checker trait + built-ins
facts-macros/     # proc-macros for facts!, tabular!, provided!
facts-mock/       # optional: trait-mock generator (feature-gated)
```

## Open design questions

1. **Proc-macro vs. declarative** for `facts!` — proc-macro gives better spans and error messages; declarative is easier to vendor. Start declarative, migrate hot paths to proc-macro once the grammar stabilizes.
2. **Async facts** — `fact!(async_thing().await => 5)` works today, but an `async_facts!` form could auto-wrap in a runtime. Defer until someone asks.
3. **Property-based checkers** — thin wrapper over `proptest` so `fact!(forall(any::<u32>()) |n| n + 0 => n)` composes naturally, or leave that to proptest directly.

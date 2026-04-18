# facts

A Rust testing library inspired by Clojure's [Midje](https://github.com/marick/midje). Assertions read left-to-right as **actual → expected**:

```rust
use facts::prelude::*;

#[test]
fn arithmetic() {
    fact!(2 + 2 => 4);
    fact!((2.0_f64).sqrt() => roughly(1.414, 0.001));
    fact!(vec![1, 2, 3] => contains(2));
}
```

Every `fact!` is a plain assertion — `cargo test`, filters, `--nocapture`, and parallelism all keep working.

## Install

Add to `Cargo.toml`:

```toml
[dev-dependencies]
facts = "0.0.1"
```

## The arrow

`fact!(actual => expected)` succeeds when `expected` (viewed as a [`Checker`](src/lib.rs)) matches `actual`. The right-hand side can be:

- **a literal** — `fact!(2 + 2 => 4)` uses the blanket `PartialEq + Debug` impl;
- **a checker constructor** — `roughly(1.414, 0.001)`, `contains(2)`, `has_prefix("Dr.")`, `is_ok()`, `is_some()`, `truthy()`, …;
- **a combinator** — `not(eq(5))`, `any_of!(1, 2, 3)`, `all_of!(has_prefix("h"), has_suffix("o"))`.

`refute!` is the negation — it fails *if* the checker matches. An optional string label in either macro surfaces in the panic message:

```rust
fact!("basic addition", 2 + 2 => 4);
refute!("not equal", 2 => 3);
```

## Grouped facts

```rust
use facts::prelude::*;

facts! { arithmetic
    fact!(2 + 2 => 4);
    fact!(5 - 3 => 2);
}

facts! { strings, "prefix and suffix checks"
    fact!("Dr. Strange" => has_prefix("Dr."));
    fact!("Dr. Strange" => has_suffix("Strange"));
}
```

`facts!` expands to a `#[test] fn`. The identifier becomes the test name; the optional string is a documentation label. A future proc-macro version will derive the name from the label directly.

## Tables

```rust
tabular! {
    fact!(a + b => sum),
    (a, b, sum),
    (1, 2, 3),
    (4, 5, 9),
    (-1, 1, 0),
}
```

Each row expands to `{ let (a, b, sum) = (1, 2, 3,); fact!(a + b => sum); }`. Column counts and types must be homogeneous — Rust type-checks the generated `let` bindings. Single-column tables need a trailing comma in the header: `(n,)`.

## Mocks for `provided!`

```rust
use facts::prelude::*;

mockable! {
    pub trait Clock as ClockMock {
        fn now(&self) -> u32;
        fn tzname(&self) -> String;
    }
}

fn greet(c: &dyn Clock) -> &'static str {
    match c.now() {
        0..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
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

`mockable!` generates the trait plus a sibling mock whose fields are `RefCell<Option<Return>>` slots — one per method. `provided!` is sugar for writing to those slots. Calling an unset slot panics. Scope of this first pass: `&self` methods with `Clone` return types. `&mut self`, generics, argument matchers, and call-count verification are on the roadmap.

## Writing your own checker

A checker is any type that implements `Checker<Target>`:

```rust
use facts::{Checker, Outcome};

pub struct Even;
pub fn even() -> Even { Even }

impl Checker<i32> for Even {
    fn check(&self, actual: &i32) -> Outcome {
        if actual % 2 == 0 {
            Outcome::pass()
        } else {
            Outcome::fail(format!("{} is not even", actual))
        }
    }
}

// fact!(4 => even());
```

The blanket impl `impl<T: PartialEq + Debug> Checker<T> for T` handles equality for any `T`. Custom checkers are distinct types, so there's no overlap — you can add as many as you want.

## Status

`0.0.1` — experimental. API may change. See [`design.md`](design.md) for the longer-form design.

## License

MIT OR Apache-2.0

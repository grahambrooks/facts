//! `facts` — a Midje-inspired testing library.
//!
//! Assertions read left-to-right: `actual => expected`. The right-hand side
//! is anything that implements [`Checker`], including any `T: PartialEq + Debug`
//! via a blanket impl, so literals work directly.
//!
//! ```no_run
//! use facts::prelude::*;
//!
//! #[test]
//! fn arithmetic() {
//!     fact!(2 + 2 => 4);
//!     fact!((2.0_f64).sqrt() => roughly(1.414, 0.001));
//!     fact!(vec![1, 2, 3] => contains(2));
//!     fact!("labeled assertion", 1 + 1 => 2);
//! }
//! ```

use std::fmt::Debug;

pub mod checkers;
pub mod mock;

pub mod prelude {
    pub use crate::checkers::*;
    pub use crate::{
        Checker, Outcome, all_of, any_of, fact, facts, mockable, provided, refute, tabular,
    };
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub passed: bool,
    pub reason: String,
}

impl Outcome {
    pub fn pass() -> Self {
        Self {
            passed: true,
            reason: String::new(),
        }
    }

    pub fn fail(reason: impl Into<String>) -> Self {
        Self {
            passed: false,
            reason: reason.into(),
        }
    }
}

pub trait Checker<T: ?Sized> {
    fn check(&self, actual: &T) -> Outcome;
}

impl<T> Checker<T> for T
where
    T: PartialEq + Debug,
{
    fn check(&self, actual: &T) -> Outcome {
        if self == actual {
            Outcome::pass()
        } else {
            Outcome::fail(format!("expected {:?}, got {:?}", self, actual))
        }
    }
}

#[macro_export]
macro_rules! fact {
    ($label:literal, $actual:expr => $expected:expr $(,)?) => {{
        let __facts_actual = $actual;
        let __facts_expected = $expected;
        let __facts_outcome = $crate::Checker::check(&__facts_expected, &__facts_actual);
        if !__facts_outcome.passed {
            panic!(
                "\nfact failed at {}:{}\n  label: {}\n  expression: {} => {}\n  {}\n",
                file!(),
                line!(),
                $label,
                stringify!($actual),
                stringify!($expected),
                __facts_outcome.reason,
            );
        }
    }};
    ($actual:expr => $expected:expr $(,)?) => {{
        let __facts_actual = $actual;
        let __facts_expected = $expected;
        let __facts_outcome = $crate::Checker::check(&__facts_expected, &__facts_actual);
        if !__facts_outcome.passed {
            panic!(
                "\nfact failed at {}:{}\n  expression: {} => {}\n  {}\n",
                file!(),
                line!(),
                stringify!($actual),
                stringify!($expected),
                __facts_outcome.reason,
            );
        }
    }};
}

#[macro_export]
macro_rules! refute {
    ($label:literal, $actual:expr => $expected:expr $(,)?) => {{
        let __facts_actual = $actual;
        let __facts_expected = $expected;
        let __facts_outcome =
            $crate::Checker::check(&__facts_expected, &__facts_actual);
        if __facts_outcome.passed {
            panic!(
                "\nrefutation failed at {}:{}\n  label: {}\n  expression: {} =not=> {}\n  but the checker matched\n",
                file!(),
                line!(),
                $label,
                stringify!($actual),
                stringify!($expected),
            );
        }
    }};
    ($actual:expr => $expected:expr $(,)?) => {{
        let __facts_actual = $actual;
        let __facts_expected = $expected;
        let __facts_outcome =
            $crate::Checker::check(&__facts_expected, &__facts_actual);
        if __facts_outcome.passed {
            panic!(
                "\nrefutation failed at {}:{}\n  expression: {} =not=> {}\n  but the checker matched\n",
                file!(),
                line!(),
                stringify!($actual),
                stringify!($expected),
            );
        }
    }};
}

/// Variadic combinator: passes if *any* inner checker passes.
#[macro_export]
macro_rules! any_of {
    ($c:expr $(,)?) => { $c };
    ($a:expr, $($rest:expr),+ $(,)?) => {
        $crate::checkers::AnyOf::new($a, $crate::any_of!($($rest),+))
    };
}

/// Variadic combinator: passes only if *all* inner checkers pass.
#[macro_export]
macro_rules! all_of {
    ($c:expr $(,)?) => { $c };
    ($a:expr, $($rest:expr),+ $(,)?) => {
        $crate::checkers::AllOf::new($a, $crate::all_of!($($rest),+))
    };
}

/// Run the same fact template over a table of rows.
///
/// Shape: `tabular!(template, (headers,), (row,), (row,), ...)`.
/// The template is any expression that references the header names;
/// each row is a tuple of values bound positionally to the headers.
///
/// ```no_run
/// use facts::prelude::*;
///
/// #[test]
/// fn addition_table() {
///     tabular! {
///         fact!(a + b => sum),
///         (a, b, sum),
///         (1, 2, 3),
///         (4, 5, 9),
///         (-1, 1, 0),
///     }
/// }
/// ```
///
/// All rows must share column count and types — the template expands
/// to one `let (headers,) = (row,);` binding per row, so a type
/// mismatch is a normal compile error. Single-column tables require a
/// trailing comma in the header tuple: `(n,)`.
#[macro_export]
macro_rules! tabular {
    (
        $body:expr,
        $headers:tt
        $(, ( $($val:expr),+ $(,)? ) )+
        $(,)?
    ) => {
        $(
            {
                let $headers = ( $($val,)+ );
                $body;
            }
        )+
    };
}

/// Group related facts under a single `#[test]` function.
///
/// ```no_run
/// use facts::prelude::*;
///
/// facts! { arithmetic
///     fact!(2 + 2 => 4);
///     fact!(5 - 3 => 2);
/// }
/// ```
///
/// An optional string label is accepted for documentation:
///
/// ```no_run
/// use facts::prelude::*;
///
/// facts! { arithmetic, "about numbers"
///     fact!(2 + 2 => 4);
/// }
/// ```
///
/// The declarative form requires an identifier because `macro_rules!`
/// cannot synthesize a fn name from a string. A future proc-macro
/// version will accept `facts! { "arithmetic" ... }` directly.
#[macro_export]
macro_rules! facts {
    ($name:ident, $label:literal $($body:tt)*) => {
        #[test]
        fn $name() {
            let _facts_label: &'static str = $label;
            $($body)*
        }
    };
    ($name:ident $($body:tt)*) => {
        #[test]
        fn $name() {
            $($body)*
        }
    };
}

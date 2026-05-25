//! Built-in checkers. Each checker is a plain value whose type implements
//! [`Checker`] for the target type(s) it understands.

use crate::{Checker, Outcome};
use std::fmt::Debug;

// ---------------------------------------------------------------------------
// eq — explicit equality (for composition; literals use the blanket impl)
// ---------------------------------------------------------------------------

pub struct Eq<T>(T);

pub fn eq<T>(value: T) -> Eq<T> {
    Eq(value)
}

impl<T: PartialEq + Debug> Checker<T> for Eq<T> {
    fn check(&self, actual: &T) -> Outcome {
        if self.0 == *actual {
            Outcome::pass()
        } else {
            Outcome::fail(format!("expected {:?}, got {:?}", self.0, actual))
        }
    }
}

// ---------------------------------------------------------------------------
// roughly — numeric tolerance
// ---------------------------------------------------------------------------

pub struct Roughly {
    target: f64,
    tolerance: f64,
}

pub fn roughly(target: f64, tolerance: f64) -> Roughly {
    Roughly { target, tolerance }
}

impl Checker<f64> for Roughly {
    fn check(&self, actual: &f64) -> Outcome {
        let diff = (actual - self.target).abs();
        if diff <= self.tolerance {
            Outcome::pass()
        } else {
            Outcome::fail(format!(
                "|{} - {}| = {} exceeds tolerance {}",
                actual, self.target, diff, self.tolerance
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// contains — slice membership
// ---------------------------------------------------------------------------

pub struct Contains<T> {
    needle: T,
}

pub fn contains<T>(needle: T) -> Contains<T> {
    Contains { needle }
}

impl<T, C> Checker<C> for Contains<T>
where
    T: PartialEq + Debug,
    C: AsRef<[T]> + Debug,
{
    fn check(&self, actual: &C) -> Outcome {
        let slice = actual.as_ref();
        if slice.iter().any(|x| x == &self.needle) {
            Outcome::pass()
        } else {
            Outcome::fail(format!("{:?} does not contain {:?}", actual, self.needle))
        }
    }
}

// ---------------------------------------------------------------------------
// string checkers — has_prefix / has_suffix / has_substring
// ---------------------------------------------------------------------------

pub struct HasPrefix(String);
pub struct HasSuffix(String);
pub struct HasSubstring(String);

pub fn has_prefix(s: impl Into<String>) -> HasPrefix {
    HasPrefix(s.into())
}
pub fn has_suffix(s: impl Into<String>) -> HasSuffix {
    HasSuffix(s.into())
}
pub fn has_substring(s: impl Into<String>) -> HasSubstring {
    HasSubstring(s.into())
}

impl<S: AsRef<str>> Checker<S> for HasPrefix {
    fn check(&self, actual: &S) -> Outcome {
        let s = actual.as_ref();
        if s.starts_with(self.0.as_str()) {
            Outcome::pass()
        } else {
            Outcome::fail(format!("{:?} does not start with {:?}", s, self.0))
        }
    }
}

impl<S: AsRef<str>> Checker<S> for HasSuffix {
    fn check(&self, actual: &S) -> Outcome {
        let s = actual.as_ref();
        if s.ends_with(self.0.as_str()) {
            Outcome::pass()
        } else {
            Outcome::fail(format!("{:?} does not end with {:?}", s, self.0))
        }
    }
}

impl<S: AsRef<str>> Checker<S> for HasSubstring {
    fn check(&self, actual: &S) -> Outcome {
        let s = actual.as_ref();
        if s.contains(self.0.as_str()) {
            Outcome::pass()
        } else {
            Outcome::fail(format!("{:?} does not contain {:?}", s, self.0))
        }
    }
}

// ---------------------------------------------------------------------------
// truthy / falsey
// ---------------------------------------------------------------------------

pub struct Truthy;
pub struct Falsey;

pub fn truthy() -> Truthy {
    Truthy
}
pub fn falsey() -> Falsey {
    Falsey
}

impl Checker<bool> for Truthy {
    fn check(&self, actual: &bool) -> Outcome {
        if *actual {
            Outcome::pass()
        } else {
            Outcome::fail("expected true, got false")
        }
    }
}

impl Checker<bool> for Falsey {
    fn check(&self, actual: &bool) -> Outcome {
        if !*actual {
            Outcome::pass()
        } else {
            Outcome::fail("expected false, got true")
        }
    }
}

// ---------------------------------------------------------------------------
// is_ok / is_err — Result
// ---------------------------------------------------------------------------

pub struct IsOk;
pub struct IsErr;

pub fn is_ok() -> IsOk {
    IsOk
}
pub fn is_err() -> IsErr {
    IsErr
}

impl<T: Debug, E: Debug> Checker<Result<T, E>> for IsOk {
    fn check(&self, actual: &Result<T, E>) -> Outcome {
        if actual.is_ok() {
            Outcome::pass()
        } else {
            Outcome::fail(format!("expected Ok, got {:?}", actual))
        }
    }
}

impl<T: Debug, E: Debug> Checker<Result<T, E>> for IsErr {
    fn check(&self, actual: &Result<T, E>) -> Outcome {
        if actual.is_err() {
            Outcome::pass()
        } else {
            Outcome::fail(format!("expected Err, got {:?}", actual))
        }
    }
}

// ---------------------------------------------------------------------------
// is_some / is_none — Option
// ---------------------------------------------------------------------------

pub struct IsSome;
pub struct IsNone;

pub fn is_some() -> IsSome {
    IsSome
}
pub fn is_none() -> IsNone {
    IsNone
}

impl<T: Debug> Checker<Option<T>> for IsSome {
    fn check(&self, actual: &Option<T>) -> Outcome {
        if actual.is_some() {
            Outcome::pass()
        } else {
            Outcome::fail("expected Some, got None")
        }
    }
}

impl<T: Debug> Checker<Option<T>> for IsNone {
    fn check(&self, actual: &Option<T>) -> Outcome {
        if actual.is_none() {
            Outcome::pass()
        } else {
            Outcome::fail(format!("expected None, got {:?}", actual))
        }
    }
}

// ---------------------------------------------------------------------------
// combinators: not, AnyOf, AllOf
// ---------------------------------------------------------------------------

pub struct Not<C>(C);

pub fn not<C>(inner: C) -> Not<C> {
    Not(inner)
}

impl<T: ?Sized, C: Checker<T>> Checker<T> for Not<C> {
    fn check(&self, actual: &T) -> Outcome {
        let inner = self.0.check(actual);
        if inner.passed {
            Outcome::fail("negated checker unexpectedly matched")
        } else {
            Outcome::pass()
        }
    }
}

pub struct AnyOf<L, R>(L, R);

impl<L, R> AnyOf<L, R> {
    pub fn new(left: L, right: R) -> Self {
        AnyOf(left, right)
    }
}

impl<T: ?Sized, L: Checker<T>, R: Checker<T>> Checker<T> for AnyOf<L, R> {
    fn check(&self, actual: &T) -> Outcome {
        let left = self.0.check(actual);
        if left.passed {
            return Outcome::pass();
        }
        let right = self.1.check(actual);
        if right.passed {
            return Outcome::pass();
        }
        Outcome::fail(format!(
            "no branch matched (left: {}; right: {})",
            left.reason, right.reason
        ))
    }
}

pub struct AllOf<L, R>(L, R);

impl<L, R> AllOf<L, R> {
    pub fn new(left: L, right: R) -> Self {
        AllOf(left, right)
    }
}

impl<T: ?Sized, L: Checker<T>, R: Checker<T>> Checker<T> for AllOf<L, R> {
    fn check(&self, actual: &T) -> Outcome {
        let left = self.0.check(actual);
        if !left.passed {
            return Outcome::fail(format!("left branch failed: {}", left.reason));
        }
        let right = self.1.check(actual);
        if !right.passed {
            return Outcome::fail(format!("right branch failed: {}", right.reason));
        }
        Outcome::pass()
    }
}

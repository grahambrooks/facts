use facts::prelude::*;

#[test]
fn literal_equality() {
    fact!(2 + 2 => 4);
    fact!("hi" => "hi");
}

#[test]
fn explicit_eq() {
    fact!(2 + 2 => eq(4));
}

#[test]
fn labeled_fact() {
    fact!("basic addition", 2 + 2 => 4);
}

#[test]
fn roughly_within_tolerance() {
    fact!((2.0_f64).sqrt() => roughly(1.414, 0.001));
}

#[test]
fn vec_contains_element() {
    fact!(vec![1, 2, 3] => contains(2));
}

#[test]
fn string_prefix_suffix_substring() {
    fact!("Dr. Strange".to_string() => has_prefix("Dr."));
    fact!("Dr. Strange" => has_suffix("Strange"));
    fact!("hello world" => has_substring("lo wo"));
}

#[test]
fn result_is_ok_and_err() {
    let ok: Result<i32, String> = Ok(5);
    let err: Result<i32, String> = Err("boom".into());
    fact!(ok => is_ok());
    fact!(err => is_err());
}

#[test]
fn option_is_some_and_none() {
    fact!(Some(3) => is_some());
    let n: Option<i32> = None;
    fact!(n => is_none());
}

#[test]
fn truthy_and_falsey() {
    fact!(1 + 1 == 2 => truthy());
    fact!(1 + 1 == 3 => falsey());
}

#[test]
fn not_combinator() {
    fact!(5 => not(eq(4)));
    fact!("world" => not(has_prefix("hello")));
}

#[test]
fn any_of_combinator() {
    fact!(2 => any_of!(1, 2, 3));
    fact!(7 => any_of!(eq(7), eq(8)));
}

#[test]
fn all_of_combinator() {
    fact!(5 => all_of!(not(eq(4)), not(eq(6))));
    fact!("hello world" => all_of!(has_prefix("hello"), has_suffix("world")));
}

#[test]
fn refute_literal() {
    refute!(2 + 2 => 5);
}

#[test]
fn labeled_refute() {
    refute!("two is not three", 2 => 3);
}

#[test]
#[should_panic(expected = "fact failed")]
fn failing_fact_panics() {
    fact!(2 + 2 => 5);
}

#[test]
#[should_panic(expected = "label: basic addition")]
fn failing_labeled_fact_shows_label() {
    fact!("basic addition", 2 + 2 => 5);
}

#[test]
#[should_panic(expected = "refutation failed")]
fn failing_refute_panics() {
    refute!(2 + 2 => 4);
}

#[test]
#[should_panic(expected = "no branch matched")]
fn any_of_failure_message() {
    fact!(99 => any_of!(1, 2, 3));
}

#[test]
#[should_panic(expected = "negated checker unexpectedly matched")]
fn not_failure_message() {
    fact!(5 => not(eq(5)));
}

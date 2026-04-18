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

fn greet_with_zone(c: &dyn Clock) -> String {
    format!("{} ({})", greet(c), c.tzname())
}

#[test]
fn single_expectation() {
    let clock = ClockMock::new();
    provided!(clock.now() => 8);
    fact!(greet(&clock) => "Good morning");
}

#[test]
fn multiple_expectations_in_one_block() {
    let clock = ClockMock::new();
    provided! {
        clock.now() => 14;
        clock.tzname() => "UTC".to_string();
    }
    fact!(greet_with_zone(&clock) => "Good afternoon (UTC)".to_string());
}

#[test]
fn tabular_over_mock() {
    tabular! {
        {
            let clock = ClockMock::new();
            provided!(clock.now() => hour);
            fact!(greet(&clock) => expected);
        },
        (hour, expected),
        (8, "Good morning"),
        (14, "Good afternoon"),
        (20, "Good evening"),
    }
}

#[test]
fn mock_via_default() {
    let clock: ClockMock = Default::default();
    provided!(clock.now() => 0);
    fact!(greet(&clock) => "Good morning");
}

#[test]
fn unrelated_unset_slot_is_fine() {
    // Only `now` is called by `greet`; leaving `tzname` unset must not panic.
    let clock = ClockMock::new();
    provided!(clock.now() => 9);
    fact!(greet(&clock) => "Good morning");
}

#[test]
#[should_panic(expected = "was called without a provided! expectation")]
fn unset_slot_panics_on_call() {
    let clock = ClockMock::new();
    let _ = clock.now();
}

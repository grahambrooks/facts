use facts::prelude::*;

#[test]
fn addition_table() {
    tabular! {
        fact!(a + b => sum),
        (a, b, sum),
        (1, 2, 3),
        (4, 5, 9),
        (-1, 1, 0),
        (0, 0, 0),
    }
}

#[test]
fn string_transform_table() {
    tabular! {
        fact!(input.to_uppercase() => expected.to_string()),
        (input, expected),
        ("hello", "HELLO"),
        ("world", "WORLD"),
        ("", ""),
    }
}

#[test]
fn single_column_table() {
    tabular! {
        fact!(n % 2 => 0),
        (n,),
        (2,),
        (4,),
        (100,),
    }
}

#[test]
fn table_with_checkers() {
    tabular! {
        fact!(value => roughly(target, 0.01)),
        (value, target),
        (1.001, 1.0),
        (2.005, 2.0),
        (3.0, 3.0),
    }
}

#[test]
fn tabular_with_refute() {
    tabular! {
        refute!(a + b => wrong),
        (a, b, wrong),
        (1, 2, 4),
        (5, 5, 11),
    }
}

facts! { grouped_table
    tabular! {
        fact!(a * b => product),
        (a, b, product),
        (2, 3, 6),
        (0, 5, 0),
        (-2, 4, -8),
    }
}

#[test]
#[should_panic(expected = "fact failed")]
fn failing_row_panics() {
    tabular! {
        fact!(a + b => sum),
        (a, b, sum),
        (1, 2, 3),
        (4, 5, 10),
    }
}

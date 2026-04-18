use facts::prelude::*;

facts! { arithmetic
    fact!(2 + 2 => 4);
    fact!(5 - 3 => 2);
    fact!(3 * 4 => 12);
}

facts! { strings, "prefix and suffix"
    fact!("Dr. Strange" => has_prefix("Dr."));
    fact!("Dr. Strange" => has_suffix("Strange"));
}

facts! { mixed_checkers
    fact!((2.0_f64).sqrt() => roughly(1.414, 0.001));
    fact!(Some(3) => is_some());
    fact!(vec![1, 2, 3] => contains(2));
    refute!(vec![1, 2, 3] => contains(99));
}

facts! { combinator_group
    fact!(5 => any_of!(1, 3, 5, 7));
    fact!(5 => all_of!(not(eq(4)), not(eq(6))));
}

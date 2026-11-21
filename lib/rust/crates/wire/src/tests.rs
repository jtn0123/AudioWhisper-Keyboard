//! `check` is what `declare!` runs on every group at compile time; here it runs on groups built
//! by hand, so a refusal shows as a panic.
use super::*;

const fn constant(name: &'static str, value: Value) -> Constant {
    Constant {
        name,
        doc: "",
        value,
    }
}

fn check_all(constants: &'static [Constant]) {
    check(&Group {
        kotlin: "MutationOp",
        form: Form::Enum,
        path: "wire::tests",
        doc: "A group built by hand.",
        constants,
    });
}

#[test]
fn distinct_codes_pass() {
    const DISTINCT: &[Constant] = &[
        constant("ADD", Value::Int(0)),
        constant("BEGINS", Value::Int(1)),
        constant("NAME", Value::Text("name")),
    ];
    check_all(DISTINCT);
}

#[test]
#[should_panic(expected = "two constants of one group share a value")]
fn two_operations_with_one_code_are_refused() {
    const SAME_CODE: &[Constant] = &[
        constant("ADD", Value::Int(0)),
        constant("BEGINS", Value::Int(0)),
    ];
    check_all(SAME_CODE);
}

#[test]
#[should_panic(expected = "two constants of one group share a value")]
fn two_operations_with_one_name_on_the_wire_are_refused() {
    const SAME_NAME: &[Constant] = &[
        constant("PRESS", Value::Text("press")),
        constant("TAP", Value::Text("press")),
    ];
    check_all(SAME_NAME);
}

#[test]
#[should_panic(expected = "printable ASCII")]
fn a_name_kotlin_would_have_to_escape_is_refused() {
    const ESCAPED: &[Constant] = &[constant("PRICE", Value::Text("$price"))];
    check_all(ESCAPED);
}

use rust_calc::{Operator, calculate};

#[test]
fn calculates_the_four_basic_operations() {
    assert_eq!(calculate(8.0, Operator::Add, 2.0).unwrap(), 10.0);
    assert_eq!(calculate(8.0, Operator::Subtract, 2.0).unwrap(), 6.0);
    assert_eq!(calculate(8.0, Operator::Multiply, 2.0).unwrap(), 16.0);
    assert_eq!(calculate(8.0, Operator::Divide, 2.0).unwrap(), 4.0);
}

#[test]
fn rejects_division_by_zero() {
    assert_eq!(
        calculate(8.0, Operator::Divide, 0.0),
        Err("Sıfıra bölünemez")
    );
}

#[test]
fn formats_results_without_a_redundant_decimal_part() {
    assert_eq!(rust_calc::format_number(4.0), "4");
    assert_eq!(rust_calc::format_number(2.5), "2,5");
}

use rust_calc::{Language, Operator, calculate, format_number};

#[test]
fn calculates_the_four_basic_operations() {
    assert_eq!(
        calculate(8.0, Operator::Add, 2.0, Language::Turkish).unwrap(),
        10.0
    );
    assert_eq!(
        calculate(8.0, Operator::Subtract, 2.0, Language::English).unwrap(),
        6.0
    );
    assert_eq!(
        calculate(8.0, Operator::Multiply, 2.0, Language::Russian).unwrap(),
        16.0
    );
    assert_eq!(
        calculate(8.0, Operator::Divide, 2.0, Language::Turkish).unwrap(),
        4.0
    );
}

#[test]
fn rejects_division_by_zero_with_i18n() {
    assert_eq!(
        calculate(8.0, Operator::Divide, 0.0, Language::Turkish),
        Err("Sıfıra bölünemez")
    );
    assert_eq!(
        calculate(8.0, Operator::Divide, 0.0, Language::English),
        Err("Cannot divide by zero")
    );
    assert_eq!(
        calculate(8.0, Operator::Divide, 0.0, Language::Russian),
        Err("Деление на ноль невозможно")
    );
}

#[test]
fn formats_results_without_a_redundant_decimal_part() {
    assert_eq!(format_number(4.0), "4");
    assert_eq!(format_number(2.5), "2,5");
}

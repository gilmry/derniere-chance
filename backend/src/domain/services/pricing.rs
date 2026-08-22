use rust_decimal::Decimal;

/// Discount percentage shown next to a démarque, rounded to the nearest
/// integer (e.g. 8,00€ → 3,20€ is 60%).
pub fn discount_percent(prix_initial: Decimal, prix_demarque: Decimal) -> i32 {
    if prix_initial <= Decimal::ZERO {
        return 0;
    }
    let ratio = (prix_initial - prix_demarque) / prix_initial;
    (ratio * Decimal::from(100)).round().try_into().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn sixty_percent_off() {
        assert_eq!(discount_percent(dec!(8.00), dec!(3.20)), 60);
    }

    #[test]
    fn zero_original_price_is_zero_percent() {
        assert_eq!(discount_percent(dec!(0), dec!(0)), 0);
    }
}

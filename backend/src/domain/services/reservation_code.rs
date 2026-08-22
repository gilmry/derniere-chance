use uuid::Uuid;

/// Generates a human-presentable pickup code ("DC-4821") for a reservation.
/// Not guaranteed unique on its own - the `code` column is UNIQUE and the
/// reservation use case retries on conflict, so this only needs to be cheap
/// and reasonably well spread, not collision-proof.
pub fn generate() -> String {
    let n = (Uuid::new_v4().as_u128() % 10_000) as u32;
    format!("DC-{n:04}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_expected_shape() {
        let code = generate();
        assert!(code.starts_with("DC-"));
        assert_eq!(code.len(), 7);
    }
}

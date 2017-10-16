extern crate rand;

pub mod network;
pub mod game;
pub mod rules;

#[cfg(test)]
mod tests {
    use game::*;

    #[test]
    fn card_compare_same_suite() {
        assert_eq!(
            Some(true),
            Card::new(CardValue::Ace, Suite::Diamonds).better_as(
                Card::new(
                    CardValue::Number7,
                    Suite::Diamonds,
                ),
                Suite::Clubs,
            )
        )
    }
}

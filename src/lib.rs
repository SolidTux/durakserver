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
        );
        assert_eq!(
            Some(false),
            Card::new(CardValue::Number10, Suite::Diamonds).better_as(
                Card::new(CardValue::King, Suite::Diamonds),
                Suite::Diamonds,
            )
        );
    }

    #[test]
    fn card_compare_trump() {
        assert_eq!(
            Some(true),
            Card::new(CardValue::Number10, Suite::Diamonds).better_as(
                Card::new(CardValue::King, Suite::Hearts),
                Suite::Diamonds,
            )
        );
        assert_eq!(
            Some(false),
            Card::new(CardValue::Number10, Suite::Hearts).better_as(
                Card::new(
                    CardValue::King,
                    Suite::Diamonds,
                ),
                Suite::Diamonds,
            )
        );
    }

    #[test]
    fn card_compare_fail() {
        assert_eq!(
            None,
            Card::new(CardValue::Number10, Suite::Hearts).better_as(
                Card::new(
                    CardValue::King,
                    Suite::Diamonds,
                ),
                Suite::Spades,
            )
        );
        assert_eq!(
            None,
            Card::new(CardValue::Jack, Suite::Hearts).better_as(
                Card::new(CardValue::Number7, Suite::Diamonds),
                Suite::Spades,
            )
        );
    }
}

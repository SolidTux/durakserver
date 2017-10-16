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

    #[test]
    fn card_display() {
        assert_eq!(
            format!("{}", Card::new(CardValue::Number10, Suite::Hearts)),
            "02"
        );
        assert_eq!(
            format!("{}", Card::new(CardValue::Jack, Suite::Spades)),
            "J3"
        );
    }

    #[test]
    fn suite_display() {
        assert_eq!(format!("{}", Suite::Diamonds), "1");
        assert_eq!(format!("{}", Suite::Hearts), "2");
        assert_eq!(format!("{}", Suite::Spades), "3");
        assert_eq!(format!("{}", Suite::Clubs), "4");
    }

    #[test]
    fn card_value_display() {
        assert_eq!(format!("{}", CardValue::Number6), "6");
        assert_eq!(format!("{}", CardValue::Number7), "7");
        assert_eq!(format!("{}", CardValue::Number8), "8");
        assert_eq!(format!("{}", CardValue::Number9), "9");
        assert_eq!(format!("{}", CardValue::Number10), "0");
        assert_eq!(format!("{}", CardValue::Jack), "J");
        assert_eq!(format!("{}", CardValue::Queen), "Q");
        assert_eq!(format!("{}", CardValue::King), "K");
        assert_eq!(format!("{}", CardValue::Ace), "A");
    }
}

//! Native, UI-independent tarot deck, weighted drawing, and relation analysis.
//! The CLI is a thin adapter; future crates and MCP transports call this API directly.

pub mod analysis;
pub mod catalog;

use chrono::{Local, Timelike};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

pub use analysis::{analyze, relate, Analysis, Relation, SuitCount};
pub use catalog::{
    card_info, cards, find_card, spreads, CardInfo, Element, Orientation, Spread, SpreadInfo, Suit,
    TimeFactor, CARD_COUNT, MAJOR_COUNT,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrawnCard {
    pub position: String,
    pub card: String,
    pub orientation: Orientation,
    pub is_major: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Draw {
    pub seed: u64,
    pub spread: Spread,
    pub spread_name: String,
    pub question: String,
    pub time_factor: TimeFactor,
    pub cards: Vec<DrawnCard>,
}

#[derive(Debug, Clone)]
pub struct DrawRequest {
    pub spread: Spread,
    pub question: String,
    pub seed: Option<u64>,
    pub time_factor: Option<TimeFactor>,
}

impl DrawRequest {
    pub fn new(spread: Spread) -> Self {
        Self {
            spread,
            question: String::new(),
            seed: None,
            time_factor: None,
        }
    }

    pub fn with_question(mut self, question: impl Into<String>) -> Self {
        self.question = question.into();
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_time_factor(mut self, time_factor: TimeFactor) -> Self {
        self.time_factor = Some(time_factor);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarotError {
    pub code: &'static str,
    pub message: String,
}

impl TarotError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_draw",
            message: message.into(),
        }
    }
}

impl fmt::Display for TarotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for TarotError {}

/// Reproduces the Python source's weights and replacement/orientation behavior.
/// Seeds are reproducible within this Rust ChaCha20 implementation, not Python MT19937.
pub fn draw(request: &DrawRequest) -> Draw {
    let seed = request.seed.unwrap_or_else(|| {
        let mut system_rng = rand::rngs::OsRng;
        system_rng.next_u64()
    });
    let factor = request
        .time_factor
        .unwrap_or_else(|| TimeFactor::from_hour(Local::now().hour()));
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut pool: Vec<u8> = (0..CARD_COUNT).collect();
    let mut chosen = Vec::with_capacity(request.spread.positions().len());

    for position in request.spread.positions() {
        let total_weight: f64 = pool
            .iter()
            .map(|&id| card_weight(id, position.major_bias, factor))
            .sum();
        let mut ticket = rng.gen::<f64>() * total_weight;
        let selected_index = pool
            .iter()
            .position(|&id| {
                ticket -= card_weight(id, position.major_bias, factor);
                ticket < 0.0
            })
            .unwrap_or(pool.len() - 1);
        let selected_id = pool.remove(selected_index);
        let selected = card_info(selected_id).expect("pool contains only valid card ids");
        let upright_probability = if position.upright_bias { 0.7 } else { 0.6 };
        let orientation = if rng.gen::<f64>() < upright_probability {
            Orientation::Upright
        } else {
            Orientation::Reversed
        };
        chosen.push(DrawnCard {
            position: position.name.to_owned(),
            card: selected.name,
            orientation,
            is_major: selected.is_major,
        });
    }

    Draw {
        seed,
        spread: request.spread,
        spread_name: request.spread.label().to_owned(),
        question: request.question.clone(),
        time_factor: factor,
        cards: chosen,
    }
}

pub fn card_weight(id: u8, major_bias: bool, factor: TimeFactor) -> f64 {
    let Some(card) = card_info(id) else {
        return 0.0;
    };
    let base = if major_bias && card.is_major {
        60.0 / 28.0
    } else {
        1.0
    };
    let boosted = factor.boosts(card.element, card.is_major);
    base * if boosted { 1.08 } else { 1.0 }
}

/// Validate untrusted JSON before allowing it into the analysis pipeline.
pub fn validate_draw(drawn: &Draw) -> Result<(), TarotError> {
    if drawn.spread_name != drawn.spread.label() {
        return Err(TarotError::invalid("spread_name does not match spread"));
    }
    let positions = drawn.spread.positions();
    if drawn.cards.len() != positions.len() {
        return Err(TarotError::invalid(format!(
            "expected {} cards, got {}",
            positions.len(),
            drawn.cards.len()
        )));
    }
    let mut seen = HashSet::with_capacity(drawn.cards.len());
    for (actual, expected) in drawn.cards.iter().zip(positions) {
        if actual.position != expected.name {
            return Err(TarotError::invalid(format!(
                "expected position {}, got {}",
                expected.name, actual.position
            )));
        }
        let card = find_card(&actual.card)
            .ok_or_else(|| TarotError::invalid(format!("unknown card: {}", actual.card)))?;
        if card.is_major != actual.is_major {
            return Err(TarotError::invalid(format!(
                "wrong is_major flag for {}",
                actual.card
            )));
        }
        if !seen.insert(card.id) {
            return Err(TarotError::invalid(format!("duplicate card: {}", actual.card)));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_unique_deck() {
        let deck = cards();
        assert_eq!(deck.len(), 78);
        assert_eq!(deck[0].name, "愚者");
        assert_eq!(deck[21].name, "世界");
        assert_eq!(deck[22].name, "权杖Ace");
        assert_eq!(deck[77].name, "星币国王");
        assert_eq!(deck.iter().filter(|card| card.is_major).count(), 22);
        let names: HashSet<_> = deck.iter().map(|card| &card.name).collect();
        assert_eq!(names.len(), 78);
    }

    #[test]
    fn seeded_draw_is_repeatable_and_unique_for_all_spreads() {
        for spread in Spread::ALL {
            let request = DrawRequest::new(spread)
                .with_seed(42)
                .with_time_factor(TimeFactor::Night)
                .with_question("事业");
            let first = draw(&request);
            assert_eq!(first, draw(&request));
            assert_eq!(first.cards.len(), spread.positions().len());
            validate_draw(&first).unwrap();
        }
    }

    #[test]
    fn inherited_positions_and_weighting() {
        let draw = draw(
            &DrawRequest::new(Spread::Horseshoe)
                .with_seed(7)
                .with_time_factor(TimeFactor::Morning),
        );
        assert_eq!(draw.cards[0].position, "远期过去");
        assert_eq!(draw.cards[6].position, "结果");
        assert_eq!(card_weight(0, true, TimeFactor::Night), (60.0 / 28.0) * 1.08);
        assert_eq!(card_weight(22, false, TimeFactor::Morning), 1.08);
        assert_eq!(TimeFactor::from_hour(5), TimeFactor::Night);
        assert_eq!(TimeFactor::from_hour(6), TimeFactor::Morning);
        assert_eq!(TimeFactor::from_hour(12), TimeFactor::Afternoon);
        assert_eq!(TimeFactor::from_hour(18), TimeFactor::Night);
    }

    #[test]
    fn tampering_is_rejected() {
        let mut result = draw(
            &DrawRequest::new(Spread::Three)
                .with_seed(1)
                .with_time_factor(TimeFactor::Night),
        );
        result.cards[1] = result.cards[0].clone();
        assert!(validate_draw(&result).is_err());
    }

    #[test]
    fn json_roundtrip_keeps_python_field_names() {
        let result = draw(
            &DrawRequest::new(Spread::Single)
                .with_seed(0)
                .with_time_factor(TimeFactor::Night),
        );
        let json = serde_json::to_string(&result).unwrap();
        let decoded: Draw = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
        assert!(json.contains("\"spread_name\""));
        assert!(json.contains("\"orientation\""));
    }
}

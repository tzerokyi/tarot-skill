//! Conservative, deterministic relation hints. They are interpretation aids,
 //! not predictions or claims that every possible pair has a unique meaning.

use serde::Serialize;

use crate::catalog::{card_info, Element, Suit};
use crate::{validate_draw, Draw, TarotError};

#[derive(Debug, Clone, Serialize)]
pub struct SuitCount {
    pub suit: Suit,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub source: &'static str,
    pub pattern: &'static str,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    pub card_count: usize,
    pub major_count: usize,
    pub major_percent: f64,
    pub major_phase: &'static str,
    pub suits: Vec<SuitCount>,
    pub dominant_suits: Vec<Suit>,
    pub absent_suits: Vec<Suit>,
    pub relations: Vec<Relation>,
}

pub fn analyze(drawn: &Draw) -> Result<Analysis, TarotError> {
    validate_draw(drawn)?;
    let cards = drawn
        .cards
        .iter()
        .map(|item| {
            crate::find_card(&item.card)
                .ok_or_else(|| TarotError::invalid("draw contains an unknown card"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let major_count = cards.iter().filter(|card| card.is_major).count();
    let major_percent = (major_count as f64 / cards.len() as f64) * 100.0;
    let major_phase = if major_percent > 50.0 {
        "turning_point"
    } else if major_percent >= 30.0 {
        "mixed"
    } else {
        "everyday_choices"
    };

    let suits: Vec<SuitCount> = Suit::ALL
        .into_iter()
        .map(|suit| SuitCount {
            suit,
            count: cards.iter().filter(|card| card.suit == Some(suit)).count(),
        })
        .collect();
    let dominant_suits = suits
        .iter()
        .filter(|entry| entry.count >= 3)
        .map(|entry| entry.suit)
        .collect();
    let absent_suits = suits
        .iter()
        .filter(|entry| entry.count == 0)
        .map(|entry| entry.suit)
        .collect();
    let relations = cards
        .windows(2)
        .filter_map(|pair| relate(pair[0].id, pair[1].id))
        .collect();

    Ok(Analysis {
        card_count: cards.len(),
        major_count,
        major_percent,
        major_phase,
        suits,
        dominant_suits,
        absent_suits,
        relations,
    })
}

/// One highest-priority evidence-based hint per ordered pair.
/// Return None for invalid ids and identical cards.
pub fn relate(left: u8, right: u8) -> Option<Relation> {
    if left == right {
        return None;
    }
    let a = card_info(left)?;
    let b = card_info(right)?;

    let pair = (left.min(right), left.max(right));
    let (source, pattern, description): (&str, &str, String) =
        if let Some(meaning) = classic_pair(pair) {
            ("classic", "transition", meaning.to_owned())
        } else if let (Some(a_num), Some(b_num)) = (a.major_number, b.major_number) {
            if a_num + b_num == 21 {
                ("B1", "tension", "大阿卡纳编号互补至 21，形成镜像张力".to_owned())
            } else if a_num < b_num {
                ("A1", "progression", "愚人之旅顺序推进，关注成长阶段".to_owned())
            } else {
                ("A1", "reflection", "愚人之旅倒序，关注尚未消化的课题".to_owned())
            }
        } else if a.suit.is_some() && a.suit == b.suit {
            let a_rank = a.rank.expect("minor card rank");
            let b_rank = b.rank.expect("minor card rank");
            if a_rank < b_rank {
                ("A2", "progression", "同花色数字向前推进".to_owned())
            } else {
                ("A2", "reflection", "同花色数字回溯，适合重新审视前一步".to_owned())
            }
        } else if a.rank.is_some_and(|rank| rank >= 11)
            || b.rank.is_some_and(|rank| rank >= 11)
        {
            ("B2", "dialogue", "宫廷牌代表角色或行动者，另一张牌提供情境".to_owned())
        } else if a.rank.is_some() && a.rank == b.rank {
            ("B3", "resonance", "相同数字跨花色，提示同阶段议题共振".to_owned())
        } else {
            let (pattern, meaning) = element_pair(a.element, b.element);
            ("A3", pattern, meaning.to_owned())
        };

    Some(Relation {
        from: a.name,
        to: b.name,
        source,
        pattern,
        description,
    })
}

fn classic_pair(pair: (u8, u8)) -> Option<&'static str> {
    match pair {
        (13, 17) => Some("结束后的疗愈"),
        (16, 17) => Some("崩塌后的重建"),
        (13, 20) => Some("方向的重新评估"),
        (18, 19) => Some("幻象被照亮"),
        (8, 15) => Some("逐步驯化阴影"),
        (1, 2) => Some("直觉与行动校时"),
        (6, 9) => Some("独处与连接的关系"),
        (4, 10) => Some("控制面对变化的考验"),
        (10, 11) => Some("选择与时机彼此回应"),
        (16, 59) => Some("旧结构彻底结束"),
        _ => None,
    }
}

fn element_pair(a: Element, b: Element) -> (&'static str, &'static str) {
    match (a, b) {
        (Element::Fire, Element::Air) | (Element::Air, Element::Fire) => {
            ("progression", "火与风互相助推")
        }
        (Element::Water, Element::Earth) | (Element::Earth, Element::Water) => {
            ("progression", "水与土彼此滋养")
        }
        (Element::Fire, Element::Water) | (Element::Water, Element::Fire) => {
            ("tension", "火与水相遇，关注热情与情绪的消耗")
        }
        (Element::Air, Element::Earth) | (Element::Earth, Element::Air) => {
            ("tension", "风与土相遇，关注思考与现实的僵持")
        }
        (Element::Fire, Element::Earth) | (Element::Earth, Element::Fire) => {
            ("dialogue", "火与土相遇，行动需要落地")
        }
        (Element::Water, Element::Air) | (Element::Air, Element::Water) => {
            ("dialogue", "水与风相遇，留意情绪化的推理")
        }
        _ => ("resonance", "相同元素互相强化"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{draw, DrawRequest, Spread, TimeFactor};

    #[test]
    fn source_coverage_and_known_combinations() {
        assert_eq!(relate(13, 17).unwrap().source, "classic");
        assert_eq!(relate(0, 21).unwrap().source, "B1");
        assert_eq!(relate(22, 23).unwrap().source, "A2");
        assert_eq!(relate(22, 36).unwrap().source, "B3");
        assert_eq!(relate(0, 3).unwrap().source, "A1");
        assert_eq!(relate(32, 48).unwrap().source, "B2");
        assert_eq!(relate(22, 37).unwrap().source, "A3");
        assert!(relate(22, 22).is_none());
        assert!(relate(78, 1).is_none());
    }

    #[test]
    fn every_spread_analyzes() {
        for spread in Spread::ALL {
            let drawn = draw(
                &DrawRequest::new(spread)
                    .with_seed(123)
                    .with_time_factor(TimeFactor::Afternoon),
            );
            let report = analyze(&drawn).unwrap();
            assert_eq!(report.card_count, spread.positions().len());
            assert_eq!(report.relations.len(), report.card_count - 1);
        }
    }
}

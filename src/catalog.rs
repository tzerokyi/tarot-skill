use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const CARD_COUNT: u8 = 78;
pub const MAJOR_COUNT: u8 = 22;

const MAJORS: [&str; 22] = [
    "愚者", "魔术师", "女祭司", "女皇", "皇帝", "教皇", "恋人", "战车",
    "力量", "隐士", "命运之轮", "正义", "倒吊人", "死神", "节制", "恶魔",
    "高塔", "星星", "月亮", "太阳", "审判", "世界",
];
const MAJOR_ELEMENTS: [Element; 22] = [
    Element::Air, Element::Air, Element::Water, Element::Earth, Element::Fire,
    Element::Earth, Element::Air, Element::Water, Element::Fire, Element::Earth,
    Element::Fire, Element::Air, Element::Water, Element::Water, Element::Fire,
    Element::Earth, Element::Fire, Element::Air, Element::Water, Element::Fire,
    Element::Fire, Element::Earth,
];
const RANKS: [&str; 14] = [
    "Ace", "二", "三", "四", "五", "六", "七", "八", "九", "十",
    "侍从", "骑士", "皇后", "国王",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Fire,
    Water,
    Air,
    Earth,
}

impl Element {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fire => "火",
            Self::Water => "水",
            Self::Air => "风",
            Self::Earth => "土",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Suit {
    Wands,
    Cups,
    Swords,
    Pentacles,
}

impl Suit {
    pub const ALL: [Suit; 4] = [
        Suit::Wands,
        Suit::Cups,
        Suit::Swords,
        Suit::Pentacles,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Wands => "权杖",
            Self::Cups => "圣杯",
            Self::Swords => "宝剑",
            Self::Pentacles => "星币",
        }
    }

    pub const fn element(self) -> Element {
        match self {
            Self::Wands => Element::Fire,
            Self::Cups => Element::Water,
            Self::Swords => Element::Air,
            Self::Pentacles => Element::Earth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Spread {
    Single,
    Three,
    Diamond,
    Moon,
    Horseshoe,
    Celtic,
}

impl Spread {
    pub const ALL: [Spread; 6] = [
        Spread::Single,
        Spread::Three,
        Spread::Diamond,
        Spread::Moon,
        Spread::Horseshoe,
        Spread::Celtic,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Three => "three",
            Self::Diamond => "diamond",
            Self::Moon => "moon",
            Self::Horseshoe => "horseshoe",
            Self::Celtic => "celtic",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Single => "单张牌",
            Self::Three => "三牌阵",
            Self::Diamond => "五牌阵",
            Self::Moon => "月亮牌阵",
            Self::Horseshoe => "马蹄形",
            Self::Celtic => "凯尔特十字",
        }
    }

    pub fn positions(self) -> &'static [Position] {
        match self {
            Self::Single => &SINGLE,
            Self::Three => &THREE,
            Self::Diamond => &DIAMOND,
            Self::Moon => &MOON,
            Self::Horseshoe => &HORSESHOE,
            Self::Celtic => &CELTIC,
        }
    }
}

impl FromStr for Spread {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|spread| spread.key() == value)
            .ok_or_else(|| format!("unknown spread: {value}"))
    }
}

impl<'de> serde::Deserialize<'de> for Spread {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        key.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeFactor {
    Morning,
    Afternoon,
    Night,
}

impl TimeFactor {
    pub const fn from_hour(hour: u32) -> Self {
        match hour {
            6..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            _ => Self::Night,
        }
    }

    pub const fn boosts(self, element: Element, major: bool) -> bool {
        match self {
            Self::Morning => matches!(element, Element::Fire | Element::Air),
            Self::Afternoon => matches!(element, Element::Water | Element::Earth),
            Self::Night => major,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Morning => "morning",
            Self::Afternoon => "afternoon",
            Self::Night => "night",
        }
    }
}

impl FromStr for TimeFactor {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "morning" => Ok(Self::Morning),
            "afternoon" => Ok(Self::Afternoon),
            "night" => Ok(Self::Night),
            _ => Err(format!("unknown time factor: {value}")),
        }
    }
}

impl<'de> serde::Deserialize<'de> for TimeFactor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let key = String::deserialize(deserializer)?;
        key.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
pub enum Orientation {
    #[serde(rename = "正位")]
    Upright,
    #[serde(rename = "逆位")]
    Reversed,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub name: &'static str,
    pub major_bias: bool,
    pub upright_bias: bool,
}

const fn position(name: &'static str, major_bias: bool, upright_bias: bool) -> Position {
    Position {
        name,
        major_bias,
        upright_bias,
    }
}

const SINGLE: [Position; 1] = [position("当前指引", true, false)];
const THREE: [Position; 3] = [
    position("过去", false, false),
    position("现在", true, false),
    position("未来", false, true),
];
const DIAMOND: [Position; 5] = [
    position("核心", true, false),
    position("根源", false, false),
    position("阻力", false, false),
    position("潜力", false, false),
    position("建议", true, true),
];
const MOON: [Position; 4] = [
    position("新月", true, false),
    position("上弦", false, false),
    position("满月", true, false),
    position("下弦", false, false),
];
const HORSESHOE: [Position; 7] = [
    position("远期过去", false, false),
    position("近期过去", false, false),
    position("当前", true, false),
    position("近期未来", false, false),
    position("外部影响", true, false),
    position("建议", false, true),
    position("结果", true, true),
];
const CELTIC: [Position; 10] = [
    position("核心", true, false),
    position("交叉", false, false),
    position("意识目标", false, false),
    position("根基过去", false, false),
    position("近期过去", true, false),
    position("近期未来", false, false),
    position("自我", false, false),
    position("环境", false, false),
    position("希望与恐惧", false, false),
    position("结果", true, true),
];

#[derive(Debug, Clone, Serialize)]
pub struct SpreadInfo {
    pub key: Spread,
    pub name: &'static str,
    pub positions: Vec<&'static str>,
}

pub fn spreads() -> Vec<SpreadInfo> {
    Spread::ALL
        .into_iter()
        .map(|spread| SpreadInfo {
            key: spread,
            name: spread.label(),
            positions: spread.positions().iter().map(|pos| pos.name).collect(),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct CardInfo {
    pub id: u8,
    pub name: String,
    pub is_major: bool,
    pub major_number: Option<u8>,
    pub suit: Option<Suit>,
    pub rank: Option<u8>,
    pub rank_name: Option<&'static str>,
    pub element: Element,
}

pub fn card_info(id: u8) -> Option<CardInfo> {
    if id < MAJOR_COUNT {
        return Some(CardInfo {
            id,
            name: MAJORS[usize::from(id)].to_owned(),
            is_major: true,
            major_number: Some(id),
            suit: None,
            rank: None,
            rank_name: None,
            element: MAJOR_ELEMENTS[usize::from(id)],
        });
    }
    if id >= CARD_COUNT {
        return None;
    }
    let index = usize::from(id - MAJOR_COUNT);
    let suit = Suit::ALL[index / 14];
    let rank_index = index % 14;
    Some(CardInfo {
        id,
        name: format!("{}{}", suit.label(), RANKS[rank_index]),
        is_major: false,
        major_number: None,
        suit: Some(suit),
        rank: Some((rank_index + 1) as u8),
        rank_name: Some(RANKS[rank_index]),
        element: suit.element(),
    })
}

pub fn cards() -> Vec<CardInfo> {
    (0..CARD_COUNT).filter_map(card_info).collect()
}

pub fn find_card(value: &str) -> Option<CardInfo> {
    if let Ok(index) = value.parse::<u8>() {
        return card_info(index);
    }
    cards().into_iter().find(|card| card.name == value)
}

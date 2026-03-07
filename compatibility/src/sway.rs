// Credits to Sway in the Rust Programming Language

use ::rand::RngExt;
use serde::Deserialize;
use serde::Serialize;

#[test]
pub fn test() {
    let mut rng = rand::rng();
    for _ in 0..1000 {
        crate::test_same(random(&mut rng));
    }
}

fn random(rng: &mut impl rand::Rng) -> FTXresponse<Trade> {
    if rng.random() {
        FTXresponse::Result(FTXresponseSuccess {
            result: Trade::random(rng),
            success: rng.random(),
        })
    } else {
        FTXresponse::Error(FTXresponseFailure {
            success: rng.random(),
            error: crate::gen_string(rng),
        })
    }
}

#[derive(bincode_2::Encode, bincode_2::Decode, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[bincode(crate = "bincode_2")]
pub enum FTXresponse<T> {
    Result(FTXresponseSuccess<T>),
    Error(FTXresponseFailure),
}

#[derive(
    bincode_2::Encode, bincode_2::Decode, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq,
)]
#[bincode(crate = "bincode_2")]
pub struct FTXresponseSuccess<T> {
    pub success: bool,
    pub result: T,
}

#[derive(bincode_2::Encode, bincode_2::Decode, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[bincode(crate = "bincode_2")]
pub struct FTXresponseFailure {
    pub success: bool,
    pub error: String,
}

#[derive(bincode_2::Encode, bincode_2::Decode, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[bincode(crate = "bincode_2")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(bincode_2::Encode, bincode_2::Decode, Serialize, Deserialize, Debug, PartialEq)]
#[bincode(crate = "bincode_2")]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: u64,
    pub liquidation: bool,
    pub price: f64,
    pub side: TradeSide,
    pub size: f64,
    pub time: String,
}

impl Trade {
    fn random(rng: &mut impl rand::Rng) -> Self {
        Self {
            id: rng.random(),
            liquidation: rng.random(),
            price: rng.random(),
            side: if rng.random() {
                TradeSide::Buy
            } else {
                TradeSide::Sell
            },
            size: rng.random(),
            time: crate::gen_string(rng),
        }
    }
}

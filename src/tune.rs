// 'the feel knobs' ... retune values to any baseline
// python returned base=30 with r=6.3 gives step1~30 dps
// step11~3 billion dps... enemy hp rides the same ladder
// so time-to-kill stays the same across whole ~10^8 power range

use serde::{Serialize, Deserialize};
use rand::{Rng, RngExt};

pub const MAX_STEP: u32 = 11;

// geometric power ladder: weap power ~= BASE * R^(step-1)
pub const POWER_BASE: f64 = 30.0;
pub const POWER_RATIO: f64 = 6.3;
pub fn step_power(step: u32) -> f64 {
    POWER_BASE * POWER_RATIO.powi(step.clamp(1, MAX_STEP) as i32 - 1)
}

// enemy base hp = step_power(stage) * HP_TTK_SECS
// (so a same-step weap needs ~this many secs to clear a normal enemy)
// rank multiplies on top
pub const HP_TTK_SECS: f64 = 3.334;
pub fn rank_mult(rank: Rank) -> f64 {
    let mut rng = rand::rng();
    match rank {
        Rank::Normal => rng.random_range(1.0..2.0),
        Rank::Elite => rng.random_range(5.0..8.0),
        Rank::Boss => rng.random_range(21.0..22.0),
        Rank::Overlord => rng.random_range(42.0..69.0),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rank { Normal, Elite, Boss, Overlord }

// in combat repetition ramp (bonus - never exponential) cap is per weapon?
pub const REP_RAMP_MIN: f64 = 0.04;
pub const REP_RAMP_MAX: f64 = 0.17;
pub const REP_CAP_MIN: f64 = 1.7;
pub const REP_CAP_MAX: f64 = 2.7;

// currency drop tiers, higher enemy type means ability to unluck rarer metals w/ gem-zone gating
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coin { Copper, Silver, Gold, Platinum }
// min stage for each drop
pub fn coin_unlock_stage(c: Coin) -> u32 {
    match c { Coin::Copper => 1, Coin::Silver => 3, Coin::Gold => 6, Coin::Platinum => 9 }
}
pub fn coin_value(c: Coin) -> f64 {
    match c { Coin::Copper => 1.0, Coin::Silver => 20.0, Coin::Gold => 400.0, Coin::Platinum => 10_000.0 }
}

pub fn gem_tier_coin(tier: u32) -> Coin {
    match tier { 0 => Coin::Copper, 1 => Coin::Silver, 2 => Coin::Gold, _ => Coin::Platinum }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Purse {
    pub copper: f64,
    pub silver: f64,
    pub gold: f64,
    pub platinum: f64,
}

impl Purse {
    pub fn get(&self, c: Coin) -> f64 {
        match c {
            Coin::Copper    =>  self.copper,
            Coin::Silver    =>  self.silver,
            Coin::Gold      =>  self.gold,
            Coin::Platinum  =>  self.platinum
        }
    }
    pub fn add(&mut self, c: Coin, n: f64) {
        match c {
            Coin::Copper    =>  self.copper += n,
            Coin::Silver    =>  self.silver += n,
            Coin::Gold      =>  self.gold += n,
            Coin::Platinum  =>  self.platinum += n
        }
    }
    pub fn net_copper(&self) -> f64 {
        self.copper * coin_value(Coin::Copper)
        + self.silver * coin_value(Coin::Silver)
        + self.gold * coin_value(Coin::Gold)
        + self.platinum * coin_value(Coin::Platinum)
    }
}

pub fn smelt_value(power: f64, step: u32) -> f64 {
    power * 0.011 + (step as f64) * 1.8 // 0.02 + ... * 5.0 ... I like 1.1% better multiply by 80% factor
}

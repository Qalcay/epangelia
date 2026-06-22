use crate::tune::{MAX_STEP, REP_CAP_MAX, REP_CAP_MIN, REP_RAMP_MAX, REP_RAMP_MIN, step_power};
use rand::{Rng, RngExt, random_range};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseStats {
    pub base_dmg: f64,
    pub attack_speed: f64,
    pub attack_sweep: f64,
    pub hit_accuracy: f64,
    pub crit_chance: f64,
    pub lucky_chance: f64,
    pub weap_durability: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Modifiers {
    pub crit_dmg: f64,
    pub rep_ramp: f64,
    pub rep_cap: f64,
    pub element: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    pub step: u32,
    pub base: BaseStats,
    pub mods: Modifiers,
    pub gem: u8,
    pub seed: u64,
}

impl Weapon {
    // starting weapon step 1 floor stats (lowest bound)
    pub fn starter() -> Weapon {
        let p = step_power(1);
        let mut w = Weapon {
            name: String::new(),
            step: 1,
            base: BaseStats {
                base_dmg: 0.22 * p,
                attack_speed: 1.334,
                attack_sweep: 1.1,
                hit_accuracy: 0.5,
                crit_chance: 0.01,
                lucky_chance: 0.51,
                weap_durability: 1.0 * p,
            },
            mods: Modifiers { crit_dmg: 2.0, rep_ramp: REP_RAMP_MIN, rep_cap: REP_CAP_MIN, element: 0 },
            gem: 0,
            seed: random_range(1..99999),
        };
        w.name = crate::name::generate(&w);
        w
    }

    pub fn better() -> Weapon {
        let mut w = Weapon {
            name: String::new(),
            step: 1,
            base: BaseStats {
                base_dmg: 50_000.00,
                attack_speed: 3.60,
                attack_sweep: 4.0,
                hit_accuracy: 0.99,
                crit_chance: 0.222,
                lucky_chance: 0.05,
                weap_durability: 10.0,
            },
            mods: Modifiers { crit_dmg: 200.0, rep_ramp: REP_RAMP_MIN, rep_cap: REP_CAP_MIN, element: 5 },
            gem: 0,
            seed: 605,
        };
        w.name = crate::name::generate(&w);
        w
    }

    /*pub fn morningstar() -> Weapon {

    }*/

    pub fn roll(step: u32, gem: u8, element: u8, rng: &mut impl Rng) -> Weapon {
        let p = step_power(step);
        let mut w = Weapon {
            name: String::new(),
            step,
            base: BaseStats {
                base_dmg: rng.random_range(0.22..1.11) * p,
                attack_speed: rng.random_range(0.22..5.0),
                attack_sweep: rng.random_range(1.0..8.0),
                hit_accuracy: rng.random_range(0.5..0.99),
                crit_chance: rng.random_range(0.01..1.00),
                lucky_chance: rng.random_range(0.01..0.51),
                weap_durability: rng.random_range(0.22..1.0) * p,
            },
            mods: Modifiers {
                crit_dmg: rng.random_range(1.1..3.6),
                rep_ramp: rng.random_range(REP_RAMP_MIN..REP_RAMP_MAX),
                rep_cap: rng.random_range(REP_CAP_MIN..REP_CAP_MAX),
                element,
            },
            gem,
            seed: rng.random(),
        };
        let _ = MAX_STEP;
        w.name = crate::name::generate(&w);
        w
    }
}

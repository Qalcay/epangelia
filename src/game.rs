use crate::fight::paper_dps;
use crate::gems::{gem_tier, outcome, Special, N_GEMS};
use crate::stat::Weapon;
use crate::tune::{coin_unlock_stage, gem_tier_coin, smelt_value, Coin, MAX_STEP, Purse};
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub purse: Purse,
    pub gems: [u32; N_GEMS],
    pub equipped: Vec<Weapon>, // up to 3 how?
    pub stash: Vec<Weapon>,
    pub stage: u32, // currently at enemy
    pub top_stage: u32, // unlock gem tier or coins
    pub clock: crate::time::Clock,
}

impl Default for Game {
    fn default() -> Self {
        let mut purse = Purse::default();
        purse.copper = 0.0;
        Game {
            purse,
            gems: [0; N_GEMS],
            equipped: vec![Weapon::starter()],
            stash: Vec::new(),
            stage: 1,
            top_stage: 1,
            clock: crate::time::Clock::default(),
        }
    }
}

impl Game {
    pub fn coin(&self, c: Coin) -> f64 { self.purse.get(c) }
    pub fn add_coin(&mut self, c: Coin, n: f64) { self.purse.add(c, n); }

    // wealth in copper only ???
    pub fn net_copper(&self) -> f64 { self.purse.net_copper() }

    //pub fn enemy_health(&self) -> f64 { self.}

    // same coin type for purchase necessary
    pub fn buy_gem(&mut self, gem: u8) -> Result<(), String> {
        let tier = gem_tier(gem);
        let coin = gem_tier_coin(tier);
        if self.top_stage < coin_unlock_stage(coin) {
            return Err(format!("{:?} gems need stage {} first", coin, coin_unlock_stage(coin)));
        }
        let price = 2.0 * (tier as f64 + 7.0);
        if self.coin(coin) < price {
            return Err(format!("need {:.0} {:?}", price, coin));
        }
        self.add_coin(coin, -price);
        self.gems[gem as usize] += 1;
        Ok(())
    }

    // roll weapon with a gem + element, live time
    // lunar 'crack_mod' (0..1) scales affin bonus strength (as timing?)
    // crack with gems preference to gain the bonus
    pub fn roll(
        &mut self,
        gem: u8,
        element: u8,
        base_step: u32,
        crack_mod: f64,
        rng: &mut impl Rng) -> Result<(Weapon, bool), String> {
            if self.gems[gem as usize] == 0 { return Err("not enough gems aaaab".into()); }
            let oc = outcome(gem, element);
            // lunar time scale up the bonus strength (rounded), preserving sign?
            let lunar_bonus = (oc.bonus_steps as f64 * crack_mod).round() as i32;
            let mut step = (base_step as i32 + lunar_bonus).clamp(1, MAX_STEP as i32) as u32;
            if oc.special == Special::QualityUp { step = (step + 1).min(MAX_STEP); }
            let mut w = Weapon::roll(step, gem, element, rng);
            if oc.special == Special::DoubleRoll {
                let w2 = Weapon::roll(step, gem, element, rng);
                if paper_dps(&w2) > paper_dps(&w) { w = w2; }
            }
            let preserved = oc.special == Special::Preserve && rng.random_bool(0.05);
            if !preserved { self.gems[gem as usize] -= 1; }
            self.stash.push(w.clone());
            Ok((w, preserved))
    }

    // smelt a stash weapon into copper
    pub fn smelt(&mut self, idx: usize) -> Result<f64, String> {
        if idx >= self.stash.len() { return Err("bad index reading ababa".into()); }
        let w = self.stash.remove(idx);
        let v = smelt_value(paper_dps(&w), w.step);
        self.add_coin(Coin::Copper, v);
        Ok(v)
    }

    // equip a stash weapon (cap 3; puts weakest into stash)
    pub fn equip(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.stash.len() { return Err("index error second version aaabbb".into()); }
        let w = self.stash.remove(idx);
        if self.equipped.len() < 3 {
            self.equipped.push(w);
        } else {
            self.equipped.sort_by(|a, b| paper_dps(a).partial_cmp(&paper_dps(b)).unwrap());
            let weakest = self.equipped.remove(0);
            self.stash.push(weakest);
            self.equipped.push(w);
        }
        Ok(())
    }

    // raw paper dps loadout ingores elemental matchup (rough disp. only)
    // real combat uses combat::effective_dps against any enemy
    pub fn raw_loadout_dps(&self) -> f64 {
        if self.equipped.is_empty() { return 0.0; }
        let share = 1.0 / self.equipped.len() as f64;
        self.equipped.iter().map(|w| paper_dps(w) * share).sum()
    }
}

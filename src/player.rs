use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::error::Category;

use crate::player;

// energy -> attack speed ... sustained_speed = clamp(regen_frac * max/cost, min, max) ?
const SPEED_MIN: f64    = 0.8;
const SPEED_MAX: f64    = 2.0;
const ENERGY_BASE: f64  = 8.0;  // pool size at 0 upgrade
const ENERGY_USED: f64  = 1.0;  // used per attack
const ENERGY_BACK: f64  = 0.12; // regen/sec = 'this' * max_samtina

//  magic -> speel uptime ... regen/sec = magic_back * magic_max
const MAGIC_BASE: f64   = 8.0;
const MAGIC_BACK: f64   = 0.1;

pub struct UpgradePlayer {
    pub player_key: &'static str,
    pub player_name: &'static str,
    pub player_blurb: &'static str,
    pub upgrade_cost: f64,
    pub upgrade_grow: f64,
    pub upgrade_lvl: f64,
}

pub const UPGRADE: &[UpgradePlayer] = &[
    UpgradePlayer {
        player_key: "",
        player_name: "",
        player_blurb: "",
        upgrade_cost: ,
        upgrade_grow: ,
        upgrade_lvl:  },
    UpgradePlayer {
        player_key: "",
        player_name: "",
        player_blurb: "",
        upgrade_cost: ,
        upgrade_grow: ,
        upgrade_lvl:  },
    UpgradePlayer {
        player_key: "",
        player_name: "",
        player_blurb: "",
        upgrade_cost: ,
        upgrade_grow: ,
        upgrade_lvl:  },
    UpgradePlayer {
        player_key: "",
        player_name: "",
        player_blurb: "",
        upgrade_cost: ,
        upgrade_grow: ,
        upgrade_lvl:  },
];

fn upgrade(key: &str) -> Option<&'static UpgradePlayer> {
    UPGRADE.iter().find(|u| u.key == key)
}

#[derive(Clone)]
pub enum CastSpell {
    BonusToElement  { element: &'static str, mult: f64 },
    PiercingResist  { fraction: f64 },
    //PierceArmor     {},
    //HealthStrip     {},
    //WeaponFrenzy   {}, // will buff attack speed wildly 20x->5x over duration... this spell is time upgraded, 20x is cap
    Empowerment     { dps_mult: f64 },
}

pub struct DefineSpell {
    pub spell_name: &'static str,
    pub spell_blurb: &'static str,
    pub spell_cost: f64,
    pub spell_dura: f64,    // secs the spell lasts
    pub spell_effect: CastSpell,
}

pub const SPELLS: &[DefineSpell] = &[
    DefineSpell {
        spell_name: "",
        spell_blurb: "",
        spell_cost: ,
        spell_dura: ,
        spell_effect:
        CastSpell:: {} },
    DefineSpell {
        spell_name: "",
        spell_blurb: "",
        spell_cost: ,
        spell_dura: ,
        spell_effect:
        CastSpell:: {} },
    DefineSpell {
        spell_name: "",
        spell_blurb: "",
        spell_cost: ,
        spell_dura: ,
        spell_effect:
        CastSpell:: {} },
];

fn use_spell(name: &str) -> Option<&'static DefineSpell> {
    SPELLS.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

#[derive(Clone)]
pub struct ActiveSpell {
    pub effect: CastSpell,
    pub remaining: f64, // seconds left
}


/* PLAYER UPGRADES */
//

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PlayerCharacter {
    #[serde(default)] pub dust: f64,                        // smelt income
    #[serde(default)] pub stat_level: HashMap<Sting, u32>,  // upgrade level
    #[serde(skip)] pub has_energy: f64,                     // transient combat state
    #[serde(skip)] pub has_magic:  f64,                     // transient
    #[serde(skip)] pub is_active: f64,                           // transient ('None' is no buff)
}

impl PlayerCharacter {
    pub fn prime(&mut self) {
        self.has_energy = self.energy_max();
        self.has_magic  = self.magic_max();
    }

    pub fn player_level(&self, key: &str) -> u32 {
        *self.stat_level.get(key).unwrap_or(&0)
    }

    fn magnitude(&self, key: &str) -> f64 {
        // level * upgrade_lvl, look up table (or 0 if null)
        match upgrade(key) {
            Some(u) => self.level(key) as f64 * u.upgrade_lvl,
            None => 0.0
        }
    }

    pub fn energy_max(&self) -> f64 { ENERGY_BASE + self.magnitude("stamina") }
    pub fn magic_max(&self) -> f64 { MAGIC_BASE + self.magnitude("mana") }
    pub fn spell_power(&self) -> f64 { 1.0 + self.magnitude("spell") }

    pub fn dps_mult(&self) -> f64 {
        let mut m = 1.0 + self.magnitude("power");
        if let Some(a) = &self.is_active {
            if let CastSpell::Empowerment { dps_mult } = a.effect { m *= dps_mult; }
        }
        m
    }

    pub fn buff_attack_speed(self) -> f64 {
        let max = self.energy_max();
        let fill = if max > 0.0 {
            (self.has_energy / max).clamp(0.0, 1.0) } else { 0.0 };
        SPEED_MIN + (SPEED_MAX - SPEED_MIN) * fill
    }

    pub fn active_effect(&self) -> Option<&CastSpell> {
        self.is_active.as_ref().map(|a| &a.effect)
    }

    pub fn effective_resist(&self, base_resist: f64) -> f64 {
        if let Some(CastSpell::PiercingResist { fraction }) = self.active_effect() {
            base_resist * (1.0 - fraction)
        } else { base_resist }
    }

    pub fn traits_tick(&mut self, dt: f64, hits: u32) {
        let smax = self.energy_max();
        self.has_energy = (
            self.has_energy
            + ENERGY_BACK
            * smax
            * dt
            - hits as f64
            * ENERGY_USED).clamp(0.0, smax);

        let mmax = self.magic_max();
        self.has_magic = (
            self.has_magic
            + MAGIC_BACK
            * mmax
            * dt).clamp(0.0, mmax);

        if let Some(a) = &mut self.is_active {
            a.remaining -= dt;
            if a.remaining <= 0.0 { self.is_active = None; }
        }
    }

    pub fn try_cast(&mut self, name: &str) -> Result<String, String> {
        let Some(def) = spell(name) else {
            return Err(format!("spellbook does not have '{name}'...", self.spell_list()));
        };
        if self.has_magic < def.spell_cost {
            return Err(format!("out of mana {:.0}/{:.0}", self.has_magic, self.spell_cost));
        }
        self.has_magic -= def.spell_cost;

        // scale the magnitude by spell power itself
        let sp = self.spell_power();
        let casted = match def.effect.clone() {
            CastSpell:: {} => CastSpell:: {} ,
            CastSpell:: {} => CastSpell:: {} ,
            CastSpell:: {} => CastSpell:: {} ,
        };
        self.is_active = Some(ActiveSpell { });
        Ok(format!("cast {}... '{}' [{:.0}s]", def.spell_name, def.spell_blurb, def.spell_dura))
    }

    pub fn try_upgrade(&mut self, key: &str) -> Result<String, String> {
        let Some(def) = upgrade(key) else {
            return Err(format!("'{key}' is not an upgrade!"));
        };
        let lvl = self.player_level(key);
        let cost = def.upgrade_cost * def.upgrade_grow.powi(lvl as i32);
        if self.dust < cost {
            return Err(format!("get {:.0} dust for '{}' Lv. {}",
                cost,
                def.spell_name,
                lvl + 1, ));
        }
        self.dust -= cost;
        *self.stat_level.entry(key.to_string()).or_insert(0) += 1;
        self.prime_caps();
        Ok(format!("{} -> Lv. {} ({} dust remaining)",
            def.spell_name,
            lvl + 1,
            self.dust as u64))
    }

    fn prime_caps(&mut self) {
        self.has_energy = self.has_energy.min(self.energy_max()).max(self.has_energy);
        self.has_magic = self.has_magic.min(self.magic_max()).max(self.has_magic);
    }

    pub fn player_sheet(&self) -> Vec<String> {
        let mut out = vec![
            format!("dust {:.0}", self.dust),
            format!("energy {:.0}/{:.0} -> attack speed {:.2}x",
                self.has_energy, self.energy_max(), self.buff_attack_speed()),
            format!("magic  {:.0}/{:.0}",
                self.has_magic, self.magic_max()),
            format!("dps amp {:.0}%",
                (self.dps_mult() - 1.0) * 100.0),
            String::new(), "upgrade: 'char upgrade <key>'".to_string(),
        ];
        for u in UPGRADE {
            let lvl = self.player_level(u.player_key);
            let cost = u.upgrade_cost * u.upgrade_grow.powi(lvl as i32);
            out.push(format!(" {:<8} Lv. {:>2}  {:<38} from {:>8.0} dust",
                u.player_key, lvl, u.player_blurb, cost));
        }
        if let Some(a) = &self.active {
            out.push(format!("{}: {:.1}s left",
                a.spell_name, a.remaining));
        }
        out
    }
}

fn spell_list() -> String {
    SPELLS.iter().map(|s| s.spell_name).collect::<Vec<_>>().join(", ")
}

fn upgrade_keys() -> String {
    UPGRADE.iter().map(|s| s.player_key).collect::<Vec<_>>().join(", ")
}

// ===========================================================================
// INTEGRATION — where this plugs into the rest of Gemforge
// ===========================================================================
//
// 1) game.rs: add `#[serde(default)] pub player: Player` to Game (and to your
//    CharacterSheet). It's a BIRD (defaulted) -> old saves load fine. Call
//    `game.player.prime()` right after `new()` and after `recall()`.
//
// 2) combat tick: effective DPS = loadout_dps * player.dps_mult() * player.attack_speed().
//    For each hit, resolve element/resist through the active spell:
//        let resist = player.effective_resist(enemy.resist_for(elem));
//        if let Some(SpellEffect::BonusElement{element, mult}) = player.active_effect() {
//            // add `loadout_dps * mult` as `element` damage this hit
//        }
//    Then once per tick: player.tick(dt, hits_this_tick);
//
// 3) smelting: when a weapon is smelted, credit dust ->  game.player.dust += dust_from(weapon);
//    (size dust to the weapon's step so high-step smelts pay more.)
//
// 4) command parser:
//        ["char"]                  => self.lines(self.game.player.char_sheet()),
//        ["char","upgrade",k]      => self.line(self.game.player.try_upgrade(k)),
//        ["cast", name]            => self.line(self.game.player.try_cast(name)),
//    (where `line` pushes the Ok/Err string to your log)

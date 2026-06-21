use crate::enemy::Enemy;
use crate::stat::Weapon;

pub fn paper_dps(w: &Weapon) -> f64 {
    let b = &w.base;
    let hit_p = b.hit_accuracy + (1.0 - b.hit_accuracy) * b.lucky_chance;
    let crit_mult = 1.0 + b.crit_chance * (w.mods.crit_dmg - 1.0);
    b.base_dmg * hit_p * crit_mult * b.attack_sweep * b.attack_speed
}

// element matchup multi... same element as enemt resist = reduc dmg
// opposite element = better dmg...
pub fn element_mult(weapon_el: u8, resist_el: u8) -> f64 {
    let raw = (weapon_el as i32 - resist_el as i32).abs();
    let d = raw.min(32 - raw);
    if d == 0 { 0.25 }
    else if d <= 2 { 0.6 }
    else if d >= 14 { 1.5 }
    else { 1.0 }
}

// core idle model... each equipped weapon contrib. an equal share
// of the attack dealt (1 weap = 1, 2 = 1/2 * 2, 3 = 1/3 * 3)
// each share scaled by weapons element->resist multi
// mono elements loadout gets blocked by matching resist
// varied loadouts always land, effective dps
pub fn effective_dps(loadout: &[Weapon], enemy: &Enemy) -> f64 {
    if loadout.is_empty() { return 0.0; }
    let share = 1.0 / loadout.len() as f64;
    loadout.iter().map(|w| {
        paper_dps(w) * share * element_mult(w.mods.element, enemy.resist_element)
    }).sum()
}

// per weapon breakdown for display (name effective contribution multipler)
pub fn breakdown(loadout: &[Weapon], enemy: &Enemy) -> Vec<(String, f64, f64)> {
    if loadout.is_empty() { return vec![]; }
    let share = 1.0 / loadout.len() as f64;
    loadout.iter().map(|w| {
        let mult = element_mult(w.mods.element, enemy.resist_element);
        (w.name.clone(), paper_dps(w) * share * mult, mult)
    }).collect()
}

pub fn time_to_kill(loadout: &[Weapon], enemy: &Enemy) -> f64 {
    let dps = effective_dps(loadout, enemy);
    if dps <= 0.0 { f64::INFINITY } else { enemy.hp_max / dps }
}

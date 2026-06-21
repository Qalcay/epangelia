// latin names here as default, swap later
// prefix   == dominant rolled stat
// root     == weapon archetype
// grandeur == power step, affixed nomen
// suffix   == element nomen

use crate::stat::Weapon;

const ROOT: [&str; 8] = [
    "Axe",
    "Sword",
    "Spear",
    "Glaive",
    "Scythe",
    "Bow",
    "Hammer",
    "Blade"
];

const GRAND: [&str; 12] = [
    "",
    "Subtle",
    "Large",
    "Humble",
    "Noble",
    "Intense",
    "Grand",
    "Excellent",
    "Supreme",
    "Divine",
    "Perfect",
    "Aethereal",
];

const SUFFIX: [Option<&str>; 8] = [
    Some("Flames"),
    Some("the Ocean"),
    Some("the Wind"),
    Some("Stone"),
    Some("the Thunder"),
    Some("Rimefrost"),
    Some("the Night"),
    Some("the Sun"),
];

/*const SUFFIX: [Option<&str>; 9] = [
    Some("Flames"),
    Some("the Ocean"),
    Some("the Wind"),
    Some("Stone"),
    Some("the Thunder"),
    Some("Rimefrost"),
    Some("the Night"),
    Some("the Sun"),
    None,
];*/

fn prefix_for(dominant: &str, step: u32) -> &'static str {
    let i = (step as usize) % 3;
    match dominant {
        "base_dmg"          => ["First", "Second", "Third"][i],
        "attack_speed"      => ["Fast", "Fierce", "Lacerating"][i],
        "crit_chance"       => ["Powerful", "Striking", "Blasting"][i],
        "attack_sweep"      => ["Greater", "Wide", "Furlong"][i],
        "hit_accuracy"      => ["Sniping", "Pinpoint", "Surgical"][i],
        "lucky_chance"      => ["Fortunate", "Lottery", "Spoils"][i],
        _                   => ["Bronze", "Steel", "Crystal"][i], // durability
    }
}

// which rolled stat is most extreme relatice to its possible range
fn dominant_stat(w: &Weapon) -> &'static str {
    let b = &w.base;
    let cands = [
        ("base_dmg", b.base_dmg / crate::tune::step_power(w.step)),
        ("attack_speed", (b.attack_speed - 0.5) / 2.0),
        ("crit_chance", (b.crit_chance - 0.02) / 0.33),
        ("attack_sweep", (b.attack_sweep - 1.0) / 3.0),
        ("hit_accuracy", (b.hit_accuracy -0.50) / 0.33),
        ("lucky_chance", b.lucky_chance / 0.20),
        ("durability", b.weap_durability / crate::tune::step_power(w.step)),
    ];
    cands.iter().fold(cands[0], |best, &c| if c.1 > best.1 { c } else { best }).0
}

pub fn generate(w: &Weapon) -> String {
    let dom = dominant_stat(w);
    let pre = prefix_for(dom, w.step);
    let root = ROOT[(w.gem as usize) % ROOT.len()];
    let grand = GRAND[w.step.clamp(1, 11) as usize];
    let suf = SUFFIX[(w.mods.element as usize) % SUFFIX.len()];

    if grand.is_empty() {
        format!("{pre} {root} {}", suf.unwrap_or(""))
    } else {
        format!("{grand} {pre} {root} of {}", suf.unwrap_or(""))
    }
}

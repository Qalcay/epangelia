use serde::{Deserialize, Serialize};

pub const N_ELEMENTS: usize = 32;
pub const N_GEMS: usize = 32;

pub struct GemTrait {
    pub name: &'static str, // name of the game for buy/crack
    pub elements: [u8; 3],  // prime, second, final (one pick?)
    pub gem_cost: u32,      // 0..3 makes it more money need
}

// 0 -> 9 gems, 1 -> 10 gems, 2 -> 7 gems, 3 -> 6 gems
pub const GEMS: &[GemTrait] = &[
    GemTrait { name: "quartz",      elements: [SUN, EARTH, AIR], gem_cost: 0 },
    GemTrait { name: "pyrite",      elements: [FIRE, EARTH, NOX], gem_cost: 0 },
    GemTrait { name: "silica",      elements: [WATER, EARTH, AIR], gem_cost: 0 },
    GemTrait { name: "amber",       elements: [FIRE, EARTH, SUN], gem_cost: 0 },
    GemTrait { name: "jasper",      elements: [NOX, FIRE, EARTH], gem_cost: 0 },
    GemTrait { name: "amethyst",    elements: [FIRE, WATER, ICE], gem_cost: 0 },
    GemTrait { name: "garnet",      elements: [EARTH, FIRE, ELEC], gem_cost: 0 },
    GemTrait { name: "onyx",        elements: [NOX, EARTH, WATER], gem_cost: 0 },
    GemTrait { name: "leucite",     elements: [ELEC, FIRE, EARTH], gem_cost: 0 },
    GemTrait { name: "ruby",        elements: [SUN, EARTH, FIRE], gem_cost: 1 },
    GemTrait { name: "azurite",     elements: [WATER, NOX, ICE], gem_cost: 1 },
    GemTrait { name: "lazurite",    elements: [WATER, EARTH, NOX], gem_cost: 1 },
    GemTrait { name: "adamite",     elements: [WATER, FIRE, EARTH], gem_cost: 1 },
    GemTrait { name: "beryl",       elements: [AIR, ICE, WATER], gem_cost: 1 },
    GemTrait { name: "pearl",       elements: [WATER, SUN, ICE], gem_cost: 1 },
    GemTrait { name: "gypsum",      elements: [WATER, ICE, EARTH], gem_cost: 1 },
    GemTrait { name: "peridot",     elements: [EARTH, SUN, FIRE], gem_cost: 1 },
    GemTrait { name: "cuprite",     elements: [EARTH, NOX, ELEC], gem_cost: 1 },
    GemTrait { name: "titanite",    elements: [ELEC, FIRE, EARTH], gem_cost: 1 },
    GemTrait { name: "sapphire",    elements: [WATER, AIR, ICE], gem_cost: 2 },
    GemTrait { name: "topaz",       elements: [SUN, AIR, ELEC], gem_cost: 2 },
    GemTrait { name: "zircon",      elements: [ELEC, SUN, EARTH], gem_cost: 2 },
    GemTrait { name: "opal",        elements: [SUN, WATER, AIR], gem_cost: 2 },
    GemTrait { name: "spinel",      elements: [FIRE, ELEC, SUN], gem_cost: 2 },
    GemTrait { name: "elbaite",     elements: [ELEC, EARTH, FIRE], gem_cost: 2 },
    GemTrait { name: "corundum",    elements: [FIRE, EARTH, SUN], gem_cost: 2 },
    GemTrait { name: "benitoite",   elements: [WATER, ELEC, AIR], gem_cost: 3 },
    GemTrait { name: "painite",     elements: [FIRE, EARTH, NOX], gem_cost: 3 },
    GemTrait { name: "jadeite",     elements: [EARTH, SUN, NOX], gem_cost: 3 },
    GemTrait { name: "bazzite",     elements: [AIR, WATER, ELEC], gem_cost: 3 },
    GemTrait { name: "emerald",     elements: [EARTH, WATER, SUN], gem_cost: 3 },
    GemTrait { name: "diamond",     elements: [SUN, ELEC, FIRE], gem_cost: 3 },
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Special { None, Preserve, DoubleRoll, QualityUp }

#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    pub bonus_steps: i32,
    pub special: Special,
}

// skill operates here
// gems have elemental affinity
// matching it grants bonus
// bonus capped +2 here, +5 eventually?
pub fn outcome(gem: u8, element: u8) -> Outcome {
    let g = gem as i32 % N_GEMS as i32;
    let e = element as i32 % N_ELEMENTS as i32;
    let affinity = (g * 7 + 3) % N_ELEMENTS as i32;
    let anti = (affinity + N_ELEMENTS as i32 / 2) % N_ELEMENTS as i32;
    let circ = |a: i32, b: i32| { let d = (a - b).abs(); d.min(N_ELEMENTS as i32 - d) };
    let da = circ(e, affinity);
    let dn = circ(e, anti);
    let bonus_steps = if da == 0 { 2 } else if da <= 2 { 1 }
        else if dn == 0 { -2 } else if dn < 2 { -1 } else { 0 };
    let h  = ((g.wrapping_mul(131) ^ e.wrapping_mul(977)) as u32) % 100;
    let special = match h {
        0..=4 => Special::Preserve,
        5..=8 => Special::DoubleRoll,
        9..=11 => Special::QualityUp,
        _ => Special::None };
    Outcome { bonus_steps, special }
}

pub fn gem_count() -> usize { GEMS.len() }
pub fn gem_id(name: &str) -> Option<usize> {
    GEMS.iter().position(|g| g.name.eq_ignore_ascii_case(name))
}

pub fn gem_name(gem: u8) -> String {
    const G: [&str; 8] = [
        "Diamond",
        "Emerald",
        "Ruby",
        "Sapphire",
        "Opal",
        "Jade",
        "Onyx",
        "Topaz"
    ];
    format!("{}", G[(gem as usize) % G.len()])
}

pub fn element_name(element: u8) -> String {
    const E: [&str; 8] = [
        "Fire",
        "Water",
        "Wind",
        "Earth",
        "Electric",
        "Ice",
        "Darkness",
        "Light"
    ];
    format!("{}", E[(element as usize) % E.len()])
}

/*pub const ELEMENTS: [&str; 8] = {
    FIRE
}*/

// determines currency used
pub fn gem_tier(gem: u8) -> u32 { (gem as u32 / 8).min(3) }

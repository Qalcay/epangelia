use serde::{Deserialize, Serialize};

pub const N_ELEMENTS: usize = 32;
pub const N_GEMS: usize = 32;

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

// determines currency used
pub fn gem_tier(gem: u8) -> u32 { (gem as u32 / 8).min(3) }

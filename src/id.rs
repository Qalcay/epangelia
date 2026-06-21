// weapon & loadout identity
// weapon signiture is hmac_sha256 over its
// canonical binary state (fixed field order, ieee-754 bit patterns)
// keyed with a game secret compiled into the binary
//  this means:
//      - signature covers the full ~10^34 rolled-state space, not the tiny name
//          table (so it cant be rainbow tabled from names alone)
//      - editing and stat by a hair changes the signature (no tamper)
//      - forging a signature requires the scret key, not just the data
// Null weapons (empty loadout) have a fixed sentinel signature and add 0
// to combat - a singleton run is 1 real weapon + 2 nulls

use crate::hash::{hex, hmac_sha256};
use crate::stat::Weapon;

const QWERTY: &[u8] = b"qwertyabcdfghijklmnopsuvxz";

fn canonical_bytes(w: &Weapon) -> Vec<u8> {
    let mut b = Vec::with_capacity(120);
    b.extend_from_slice(&(w.step as u64).to_be_bytes());
    b.extend_from_slice(&(w.gem as u64).to_be_bytes());
    b.extend_from_slice(&(w.mods.element as u64).to_be_bytes());
    b.extend_from_slice(&w.seed.to_be_bytes());
    // floats as raw ieee-754 big endian bits (not as decimal text) for exact round-trip
    for f in [
        w.base.base_dmg,
        w.base.attack_speed,
        w.base.attack_sweep,
        w.base.hit_accuracy,
        w.base.crit_chance,
        w.base.lucky_chance,
        w.base.weap_durability,
        w.mods.crit_dmg,
        w.mods.rep_ramp,
        w.mods.rep_cap,
    ] {
        b.extend_from_slice(&f.to_bits().to_be_bytes());
    }
    b
}


pub fn weapon_sig(w: &Weapon) -> String {
    hex(&hmac_sha256(QWERTY, &canonical_bytes(w)))
}


pub fn short_sig(w: &Weapon) -> String {
    weapon_sig(w)[..12].to_string()
}


pub fn null_sig() -> String {
    hex(&hmac_sha256(QWERTY, b"NULL_WEAPON_SLOT"))
}


pub fn loadout_sig(loadout: &[Weapon]) -> String {
    let mut sigs: Vec<String> = (0..3).map(|i| {
        loadout.get(i).map(weapon_sig).unwrap_or_else(null_sig)
    }).collect();
    sigs.sort();
    let joined = sigs.join("|");
    hex(&hmac_sha256(QWERTY, joined.as_bytes()))
}


pub fn loadout_short(loadout: &[Weapon]) -> String {
    loadout_sig(loadout)[..16].to_string()
}


pub fn verify_weapon(w: &Weapon, claimed: &str) -> bool {
    let actual = weapon_sig(w);
    if actual.len() != claimed.len() { return false; }
    let mut diff = 0u8;
    for (a, c) in actual.bytes().zip(claimed.bytes()) { diff |= a ^ c; }
    diff == 0
}

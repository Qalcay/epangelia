// latin names here as default, swap later
// prefix   == dominant rolled stat
// root     == weapon archetype
// grandeur == power step, affixed nomen
// suffix   == element nomen

use crate::stat::Weapon;

const ROOT: [&[&str]; 8] = [
    &["Axe", "Mace", "Hatchet", "Cleaver"],
    &["Sword", "Dagger", "Saw", "Sabre", "Knife", "Okina"],
    &["Spear", "Pickaxe", "Lance"],
    &["Glaive", "Chakram", "Throwing Star"],
    &["Scythe", "Falchion", "Scimitar"],
    &["Bow", "Staff", "Wand", "Crossbow", "Orb"],
    &["Hammer", "Club", "Mallet"],
    &["Blade", "Machete", "Cutlass", "Katana"]
];

const GRAND: [&[&str]; 12] = [
    &["Le", "Broken", "Damaged"],
    &["Subtle", "Simple", "Dull", "Pathetic", "Shabby", "Crudy", "Easy", "Dented", "Lesser", "Meager", "Worn", "Fickle"],
    &["Large", "Fair", "Common", "Good", "Better", "Nice"],
    &["Humble", "Not The Best", "Honest", "Fine", "Wild", "Hardy", "Tried"],
    &["Noble", "Adept", "Spurious", "Sublime"],
    &["Intense", "Final", "Greater"],
    &["Grand", "The Best", "Regal", "Eminent"],
    &["Excellent", "Superb", "Resplendant"],
    &["Supreme", "Incredible", "Exalted"],
    &["Divine", "Blessed", "Saintly"],
    &["Perfect", "Flawless", "Unique", "Immaculate", "Transcient", "Pristine"],
    &["Aethereal", "Primordial", "Ungodly", "Heavenly", "Empyrael", "Hyperion"],
];

const SUFFIX: [&[&str]; 8] = [
    &["Flames", "the Hearth", "Cinder", "Burning", "the Firestorm", "Ash"],
    &["the Ocean", "the Mist", "Rainstorms", "the River", "Aquarius", "Floods"],
    &["the Wind", "the Gale", "the Air", "Swiftness", "the Windstorm", "Flow"],
    &["Stone", "Crags", "the Land", "the Karyst", "the Mountain", "the Valley"],
    &["the Thunder", "Blitz", "Craze", "Static", "Lightning", "Sparking", "Stunning"],
    &["Rimefrost", "Solitude", "Freezing", "the Cold", "Permafrost", "Weakness"],
    &["the Night", "the Dim", "Darkness", "the Dusk", "Black", "Malice"],
    &["the Sun", "the Light", "Lumination", "Brilliance", "the Dawn", "White", "Purity"],
];

/*const SUFFIX: [Option<&str>; 8] = [
    Some("Flames"),
    Some("the Ocean"),
    Some("the Wind"),
    Some("Stone"),
    Some("the Thunder"),
    Some("Rimefrost"),
    Some("the Night"),
    Some("the Sun"),
];*/

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

// which rolled stat is most extreme relative to its possible range
fn dominant_stat(w: &Weapon) -> &'static str {
    let b = &w.base;
    let cands = [
        ("base_dmg", b.base_dmg / crate::tune::step_power(w.step)),
        ("attack_speed", (b.attack_speed - 0.22) / 5.0),
        ("crit_chance", (b.crit_chance - 0.01) / 0.99),
        ("attack_sweep", (b.attack_sweep - 1.0) / 8.0),
        ("hit_accuracy", (b.hit_accuracy - 0.50) / 0.99),
        ("lucky_chance", (b.lucky_chance - 0.01) / 0.51),
        ("durability", b.weap_durability / crate::tune::step_power(w.step)),
    ];
    cands.iter().fold(cands[0], |best, &c| if c.1 > best.1 { c } else { best }).0
}

pub fn generate(w: &Weapon) -> String {
    let dom = dominant_stat(w);
    let pre = prefix_for(dom, w.step);
    let root_pool = ROOT[w.gem.clamp(1, 8) as usize];
    let root_name = root_pool[(w.seed % root_pool.len() as u64) as usize];
    let grand_pool = GRAND[w.step.clamp(1, 11) as usize];
    let grand_name = grand_pool[(w.seed % grand_pool.len() as u64) as  usize];
    let suf_pool = SUFFIX[w.mods.element.clamp(0, 7) as usize];
    let suf_name = suf_pool[(w.seed % suf_pool.len() as u64) as usize];

    if grand_name.is_empty() {
        format!("{pre} {root_name} {suf_name}")
    } else {
        format!("{grand_name} {pre} {root_name} of {suf_name}")
        //format!("{grand_name} {pre} {root_name} of {}", suf.unwrap_or(""))
    }
}

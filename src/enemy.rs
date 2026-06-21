use crate::tune::{coin_unlock_stage, rank_mult, step_power, Coin, Rank, HP_TTK_SECS};
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub stage: u32,
    pub rank: Rank,
    pub hp_max: f64,
    pub resist_element: u8,
}

impl Enemy {
    pub fn spawn(stage: u32, rank: Rank, rng: &mut impl Rng) -> Enemy {
        let hp = step_power(stage) * HP_TTK_SECS * rank_mult(rank);
        let resist = rng.random_range(0..32u8);
        Enemy {
            name: compose_name(rank, resist, rng),
            stage,
            rank,
            hp_max: hp,
            resist_element: resist,
        }
    }

    // currency to drop on kill, gated by stage,
    // better stages can drop rarer metals, lower only grants copper
    pub fn drop(&self, rng: &mut impl Rng) -> (Coin, f64) {
        let coin = if self.stage >= coin_unlock_stage(Coin::Platinum) && rng.random_bool(0.1) {
            Coin::Platinum
        } else if self.stage >= coin_unlock_stage(Coin::Gold) && rng.random_bool(0.2) {
            Coin::Gold
        } else if self.stage >= coin_unlock_stage(Coin::Silver) && rng.random_bool(0.4) {
            Coin::Silver
        } else {
            Coin::Copper
        };
        let amount = (self.hp_max.sqrt() * rank_mult(self.rank) * 0.3).max(1.0);
        (coin, amount)
    }
}

const SPEC: [Option<&str>; 8] = [
    Some("Ashen"),
    Some("Brine"),
    Some("Gust"),
    Some("Clay"),
    Some("Shock"),
    Some("Frost"),
    Some("Shaded"),
    Some("Shiny"),
];

/*const SPEC: [Option<&str>; 9] = [
    Some("Ashen"),
    Some("Brine"),
    Some("Gust"),
    Some("Clay"),
    Some("Shock"),
    Some("Frost"),
    Some("Shaded"),
    Some("Shiny"),
    None,
];*/

/*const SPEC: [&str; 8] = [
    "Ashen",
    "Brine",
    "Gust",
    "Clay",
    "Shock",
    "Frost",
    "Shaded",
    "Shiny"
];*/

const MOB_NORMAL: [&str; 16] = [
    "Ghost",
    "Skeleton",
    "Zombie",
    "Robot",
    "Frog",
    "Rat",
    "Hunter",
    "Chimp",
    "Steed",
    "Vagrant",
    "Crab",
    "Wolf",
    "Archer",
    "Sellsword",
    "Mage",
    "Spider",
];

/*const MOB_NORMAL: [&str; 6] = [
    "Gui", // Gui is soul stuck in the mortal realm, 'hungry ghost'
    "Xiao", // Xiao are small feral mountain demons dwell in caves, brutish
    "Mogwai", // Mogwai is a cantonese demon that relies on the misfortune of others
    "Menehune", // Menehune little creatures that cause mischief
    "n5",
    "n6"
];*/

const MOB_ELITE: [&str; 16] = [
    "Lion",
    "Snake",
    "Android",
    "Locusts",
    "Imp",
    "Troll",
    "Shark",
    "Rider",
    "Mercenary",
    "Bear",
    "Witch",
    "Arachnid",
    "Leaper",
    "Dwarf",
    "Toad",
    "Eel",
];

const MOB_BOSS: [&str; 16] = [
    "Cyborg",
    "Serpent",
    "Wizard",
    "Protean",
    "Champion",
    "Ninja",
    "Mage",
    "Werewolf",
    "Serpent",
    "Serpent",
    "Mage",
    "Ninja",
    "Werewolf",
    "Champion",
    "Protean",
    "Wizard",
];

const MOB_OVERLORD: [&str; 16] = [
    "Ravenous Being",
    "Kraken",
    "Demon",
    "Tumor",
    "Giant",
    "Dragon",
    "Sage",
    "Sage",
    "Dragon",
    "Tumor",
    "Giant",
    "Demon",
    "Kraken",
    "Giant",
    "Dragon",
    "Dragon",
];

fn compose_name(rank: Rank, resist: u8, rng: &mut impl Rng) -> String {
    let spec = SPEC[(resist as usize) % SPEC.len()];
    let pool = match rank {
        Rank::Normal => &MOB_NORMAL,
        Rank::Elite => &MOB_ELITE,
        Rank::Boss => &MOB_BOSS,
        Rank::Overlord => &MOB_OVERLORD,
    };
    let noun = pool[rng.random_range(0..pool.len())];
    let prefix = match rank {
        Rank::Normal => "",
        Rank::Elite => "Mighty",
        Rank::Boss => "Colossal",
        Rank::Overlord => "Chaos",
    };
    format!("{prefix} {} {noun}", spec.unwrap_or(""))
}

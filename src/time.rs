use serde::{Deserialize, Serialize};

pub const LUNATION_CYCLE: f64 = 27.334;
pub const DAYS_WITHIN_MONTH: u64 = 30;
pub const MONTHS_WITHIN_YEAR: u64 = 12;
pub const DAYS_WITHIN_YEAR: u64 = DAYS_WITHIN_MONTH * MONTHS_WITHIN_YEAR;

const GRECO_MONTHS: [&str; 12] = [
    "Gamelion", "Anthesterion", "Elaphebolion", "Mounichion",
    "Thargelion", "Skirophorion", "Hekatombaion", "Metageitnion",
    "Boedromion", "Pyanespion", "Maimakterion", "Poseidon",
];

const PHASE_NAMES: [&str; 8] = [
    "Novel", "Crescent Forming", "Formed Half", "Gibbous Forming",
    "Complete", "Gibbous Deforming", "Deformed Half", "Crescent Deforming",
];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub struct Clock {
    // total in-game days elapsed since epoch
    pub day: u64,
    // fractional prog. thru the curr day (adv. with the tick loop)
    pub frac: f64,
}

impl Clock {
    pub fn advance(&mut self, days: f64) {
        self.frac += days;
        let whole = self.frac.floor();
        self.day += whole as u64;
        self.frac -= whole;
    }

    fn day_of_month(&self) -> u64 { self.day % DAYS_WITHIN_MONTH + 1 }
    fn month(&self) -> usize { ((self.day / DAYS_WITHIN_MONTH) % MONTHS_WITHIN_YEAR) as usize }
    fn year(&self) -> u64 { self.day / DAYS_WITHIN_YEAR + 1}

    pub fn lunation_percent(&self) -> f64 {
        let abs_period = self.day as f64 + self.frac;
        let abs_position = (abs_period % LUNATION_CYCLE) / LUNATION_CYCLE;
        abs_position * 100.0
        //let month_progression = (self.day_of_month() as f64 - 1.0) + self.frac;
        //(month_progression / DAYS_WITHIN_MONTH as f64) * 100.0
    }

    pub fn illuminate_state(&self) -> f64 {
        let pos = self.lunation_percent() / 100.0;
        (1.0 - (2.0 * std::f64::consts::PI * pos).cos()) / 2.0 * 100.0
    }

    pub fn label_phase(&self) -> &'static str {
        let q = self.illuminate_state() / 100.0;
        let waxed = self.lunation_percent() < 50.0; // first half of cycle = rising
        if q < 0.02 { return PHASE_NAMES[0]; }
        if q > 0.98 { return PHASE_NAMES[4]; }
        if q < 0.5 {
            return if waxed { PHASE_NAMES[1] } else { PHASE_NAMES[7] };
        }
        if (q - 0.5).abs() < 0.06 {
            return if waxed { PHASE_NAMES[2] } else { PHASE_NAMES[6] };
        }
        if waxed { PHASE_NAMES[3] } else { PHASE_NAMES[5] }
        //PHASE_NAMES[q.min(7)]
    }

    /*pub fn label_phase(&self) -> &'static str {
        let q = (self.lunation_percent() / 125.0) as usize;
        PHASE_NAMES[q.min(7)]
    }*/

    pub fn date_string(&self) -> String {
        format!("\t{} {}, {} | Moon: {:.1}% ({})",
            self.day_of_month(),
            GRECO_MONTHS[self.month()],
            (self.year() + 1332),
            self.illuminate_state(),
            self.label_phase())
    }



    // effective multi (0.0..1.0) for an item whose hidden favor moon
    // percentage is a favorite? Cicular closeness: cycle wraps,
    // 99% and 1% are near each other...
    // ...peak is 1.0 at favored %, 0.0 at its opposition
    pub fn affinity(&self, favor_perc: f64) -> f64 {
        let raw = (self.lunation_percent() - favor_perc).abs();
        let dist = raw.min(100.0 - raw); // 0..50?
        1.0 - (dist / 50.0)
    }

    pub fn percent_affinity(seed: u64) -> f64 {
        let h = seed.wrapping_mul(4693517212).wrapping_add(1224181172);
        (h % 1000) as f64 / 10.0 // 0.0..99.9
    }

    pub fn lunation_buff(&self, seed: u64) -> f64 {
        self.affinity(Self::percent_affinity(seed))
    }





    /*
    pub fn illumination(&self) -> f64 {
        let d = self.day_of_month() as f64;
        (1.0 - (2.0 * std::f64::consts::PI * (d - 1.0) / DAYS_WITHIN_MONTH as f64).cos()) / 2.0
    }

    pub fn at_moon_phase(&self) -> &'static str {
        let i = self.illumination();
        let d = self.day_of_month() as f64;
        let waxing = (d - 1.0) / DAYS_WITHIN_MONTH as f64; // < 0.5;
        if i < 0.03 { return "Novel"; }
        if i > 0.97 { return "Complete"; }
        if i < 0.5 { return if waxing { "Crescent Forming" } else { "Crescent Waning" }; }
        if (i - 0.5).abs() < 0.06 { return if waxing { "Formed Half" } else { "Deformed Half" }; }
        if waxing { "Gibbous Forming" } else { "Gibbous Deforming" }
    }

    pub fn date_string(&self) -> String {
        format!("{} {}, Year {} - {} Moon",
            self.day_of_month(), GRECO_MONTHS[self.month()], self.year(), self.at_moon_phase())
    }

    // lunar multi on a gem affin bonus at crack  time
    // light affinity peak full moon, dark affin peak at new moon
    // returns 0.0->1.0 and multiplies the magnitude of the affinity bonus_steps
    pub fn crack_modifier(&self, gem: u8) -> f64 {
        let illum = self.illumination();
        let night_philia = (gem as u32 * 13 + 5) % 3 == 0;
        if night_philia { 1.0 - illum } else { illum }
    }

    pub fn is_night_phile(gem: u8) -> bool { (gem as u32 * 13 + 5) % 3 == 0 }
    pub fn _at_moon_phase() -> &'static [&'static str; 8] { &PHASE_NAMES } */
}

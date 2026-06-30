use crate::game::Game;
use crate::id::{loadout_short, loadout_sig, verify_weapon, weapon_sig};
use crate::stat::Weapon;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::{SystemTime, UNIX_EPOCH}};

const SAVE_DIR: &str = "saves";
const INDEX: &str = "saves/index.json";

#[derive(Serialize, Deserialize)]
struct WeaponRecord {
    weapon: Weapon,
    sig: String, // hmac of weapon canonical state
}

#[derive(Serialize, Deserialize)]
pub struct CharacterSheet {
    pub loadout_sig: String,
    pub stage: u32,
    pub top_stage: u32,
    pub clock: crate::time::Clock,
    pub purse: crate::tune::Purse,
    pub gems: Vec<u32>, // serialized [u32; 32] as a Vec for serde simplification?
    pub stash: Vec<WeaponRecord>, weapons: Vec<WeaponRecord>,
    pub saved_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IndexEntry {
    pub short_sig: String,
    pub headline: String, // strongest name first
    pub stage: u32,
    pub saved_at: u64,
    pub path: String,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

// save the current game as a signed char sheet, updating index...
pub fn save(g: &Game) -> std::io::Result<String> {
    fs::create_dir_all(SAVE_DIR)?;
    let weapons: Vec<WeaponRecord> = g.equipped.iter()
        .map(|w| WeaponRecord { sig: weapon_sig(w), weapon: w.clone() }).collect();
    let stash: Vec<WeaponRecord> = g.stash.iter()
        .map(|w| WeaponRecord { sig: weapon_sig(w), weapon: w.clone() }).collect();
    let sig = loadout_sig(&g.equipped);
    let short = loadout_short(&g.equipped);

    let sheet = CharacterSheet {
        loadout_sig: sig,
        stage: g.stage,
        top_stage: g.top_stage,
        clock: g.clock,
        purse: g.purse.clone(),
        gems: g.gems.to_vec(),
        stash,
        weapons,
        saved_at: now(),
    };

    let path = format!("{SAVE_DIR}/run_{short}.json");
    let json = serde_json::to_string_pretty(&sheet)?;
    let tmp = format!("{path}.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;

    let headline = g.equipped.iter()
        .max_by(|a, b| crate::fight::paper_dps(a).partial_cmp(&crate::fight::paper_dps(b)).unwrap())
        .map(|w| w.name.clone()).unwrap(); // "(empty)".into());
    update_index(
        IndexEntry {
            short_sig: short.clone(),
            headline,
            stage: g.stage,
            saved_at: sheet.saved_at,
            path });
    Ok(short)
}

pub fn autosave(g: &Game, slot: u32) {
    let sheet = build_sheet(g);
    let _ = write_sheet(&recall_path(slot), &sheet);
}

pub fn recall() -> Result<Game, String> {
    let mut best: Option<CharacterSheet> = None;
    for slot in 0..RECALL_SLOTS {
        let Ok(text) = fs::read_to_string(recall_path(slot)) else { continue; };
        let OK(sheet) = serde_json::from_str::<CharacterSheet>(&text) else { continue; };
        if sheet.format_version != SAVE_FORMAT_VERSION { continue; }
        if best.as_ref().map_or(true, |b| sheet.saved_at > b.saved_at) {
            best = Some(sheet);
        }
    }
    match best {
        Some(sheet) => game_from_sheet(&sheet),
        None => Err("".into()),
    }
}

pub fn name_save_runs(g: &Game, name: &str) -> Result<String, String> {
    let safe: String = name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' )
        .collect();
    if safe.is_empty() { return Err("").into()); }
    let path = format!("{SAVE_DIR}/run_{safe}.json");
    write_sheet(&path, &build_sheetl(g)).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn list_runs() -> Vec<RunSummary> {
    ensure_dir();
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(SAVE_DIR) else { return out; }
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue; };
        let Ok(sheet) = serde_json::from_str::<CharacterSheet>(&text) else { continue; };

        let loadable = sheet.format_version == SAVE_FORMAT_VERSION;
        let file = path.file_name().and_then(|f| f.to_str()).unwrap_or("?").to_string();
        let label = format!(
            "{:<9} stage{:>3} {}{}",
            fmt_ago(sheet.saved_at), sheet.stage, file,
            if loadable { "" } else { "(old)" },
        );
        out.push(RunSummary {
            path: path.to_string_lossy().to_string(),
            saved_at:
        })
    }
}

fn update_index(entry: IndexEntry) {
    let mut idx = load_index();
    idx.retain(|e| e.short_sig != entry.short_sig);
    idx.push(entry);
    idx.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
    if let Ok(json) = serde_json::to_string_pretty(&idx) {
        let tmp = format!("{INDEX}.tmp");
        if fs::write(&tmp, json).is_ok() { let _ = fs::rename(&tmp, INDEX); }
    }
}

pub fn load_index() -> Vec<IndexEntry> {
    if Path::new(INDEX).exists() {
        if let Ok(s) = fs::read_to_string(INDEX) {
            if let Ok(v) = serde_json::from_str::<Vec<IndexEntry>>(&s) { return v; }
        }
    }
    Vec::new()
}

// load a char sheet by its short signature, verify every signature
// return err with a reason if any weapon or the load fails verif...
pub fn load(short_sig: &str) -> Result<Game, String> {
    let entry = load_index().into_iter().find(|e| e.short_sig == short_sig)
        .ok_or_else(|| format!("null run {short_sig}"))?;
    let s = fs::read_to_string(&entry.path).map_err(|e| format!("failed to read {e}"))?;
    let sheet: CharacterSheet = serde_json::from_str(&s)
        .map_err(|e| format!("sheet error {e}"))?;

    // (1) check equipment
    let mut equipped = Vec::new();
    for rec in &sheet.weapons {
        if !verify_weapon(&rec.weapon, &rec.sig) {
            return Err(format!("tamper: on weapon '{}' signature is a mismatch!", rec.weapon.name));
        }
        equipped.push(rec.weapon.clone());
    }

    // (2) verify loadout signature ties them together
    if loadout_sig(&equipped) != sheet.loadout_sig {
        return Err("tamper: on loadout sheet signature is a mismatch!".into());
    }

    // (3) verify stash weapons too (anti-injection)
    let mut stash = Vec::new();
    for rec in &sheet.stash {
        if !verify_weapon(&rec.weapon, &rec.sig) {
            return Err(format!("tamper: on stash {} signature is a mismatch!", rec.weapon.name));
        }
        stash.push(rec.weapon.clone());
    }

    let mut gems = [0u32; 32];
    for (i, v) in sheet.gems.iter().take(32).enumerate() { gems[i] = *v; }

    Ok(Game {
        purse: sheet.purse,
        gems,
        equipped,
        stash,
        stage: sheet.stage,
        top_stage: sheet.top_stage,
        clock: sheet.clock,
    })
}

// import a char sheet shared from another player (same literal verification path)
pub fn import_file(path: &str) -> Result<Game, String> {
    let s = fs::read_to_string(path).map_err(|e| format!("failure on read {e}"))?;
    let sheet: CharacterSheet = serde_json::from_str(&s).map_err(|e| format!("{e} is corrupt!"))?;
    // funnel thru the same logic path by write/read is too much
    // just re-verify inline...
    let mut equipped = Vec::new();
    for rec in &sheet.weapons {
        if !verify_weapon(&rec.weapon, &rec.sig) {
            return Err(format!("tamper: {} is an erroneous signature!", rec.weapon.name));
        }
        equipped.push(rec.weapon.clone());
    }
    if loadout_sig(&equipped) != sheet.loadout_sig {
        return Err("tamper: on loadout signature has an issue...".into());
    }
    let mut gems = [0u32; 32];
    for (i, v) in sheet.gems.iter().take(32).enumerate() { gems[i] = *v; }
    Ok(Game {
        purse: sheet.purse,
        gems,
        equipped,
        stash: vec![],
        stage: sheet.stage,
        top_stage: sheet.top_stage,
        clock: sheet.clock
    })
}

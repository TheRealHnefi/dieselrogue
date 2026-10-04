//! Save-on-quit: a single save slot next to settings.json. The file holds the game
//! state as JSON, zlib-compressed and then scrambled with a fixed keystream. That's
//! no protection against a determined cheat, but it isn't readable or casually
//! editable either, and zlib's checksum rejects a damaged file.

use std::fs;
use std::io::{Read, Write};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use crate::{World, GameLog};

const SAVE_PATH: &str = "savegame.dat";
const MAGIC: &[u8; 4] = b"DRSV";
/// Bump when the saved layout changes; saves from other versions are refused.
const FORMAT_VERSION: u32 = 2; // 2: log entries carry a LogKind tint
const SCRAMBLE_KEY: u64 = 0x5EED_D1E5_E1D0_6E75;

/// Everything a run needs to continue, beyond what the world itself holds.
#[derive(serde::Deserialize)]
pub struct SaveData {
    pub world: World,
    pub log: GameLog,
    pub turn: u32,
    pub seed: u64,
}

/// Borrowing twin of `SaveData`, so saving needn't clone the world.
#[derive(serde::Serialize)]
struct SaveRef<'a> {
    world: &'a World,
    log: &'a GameLog,
    turn: u32,
    seed: u64,
}

pub fn exists() -> bool {
    std::path::Path::new(SAVE_PATH).is_file()
}

pub fn delete() {
    let _ = fs::remove_file(SAVE_PATH);
}

pub fn save(world: &World, log: &GameLog, turn: u32, seed: u64) -> Result<(), String> {
    save_to(SAVE_PATH, world, log, turn, seed)
}

pub fn load() -> Result<SaveData, String> {
    load_from(SAVE_PATH)
}

fn save_to(path: &str, world: &World, log: &GameLog, turn: u32, seed: u64) -> Result<(), String> {
    let json = serde_json::to_vec(&SaveRef { world, log, turn, seed }).map_err(|e| e.to_string())?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json).map_err(|e| e.to_string())?;
    let mut body = encoder.finish().map_err(|e| e.to_string())?;
    scramble(&mut body);

    let mut file = Vec::with_capacity(body.len() + 8);
    file.extend_from_slice(MAGIC);
    file.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    file.extend_from_slice(&body);
    // Write beside and rename, so a crash mid-save can't leave a half-written file.
    let tmp = format!("{}.tmp", path);
    fs::write(&tmp, &file).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn load_from(path: &str) -> Result<SaveData, String> {
    let file = fs::read(path).map_err(|e| e.to_string())?;
    if file.len() < 8 || &file[0..4] != MAGIC {
        return Err("not a DieselRogue save".to_string());
    }
    let version = u32::from_le_bytes([file[4], file[5], file[6], file[7]]);
    if version != FORMAT_VERSION {
        return Err(format!("made by an incompatible version ({})", version));
    }
    let mut body = file[8..].to_vec();
    scramble(&mut body);
    let mut json = Vec::new();
    ZlibDecoder::new(&body[..]).read_to_end(&mut json).map_err(|_| "damaged".to_string())?;
    let mut data: SaveData = serde_json::from_slice(&json).map_err(|e| e.to_string())?;
    data.world.restore_after_load();
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rltk::Point;
    use crate::{ActionId, ActorAI, AlertLevel, AI, CombatTactic, Direction, Item, ItemKind, Profile};

    /// A scratch save path of its own per test, so tests never touch the real slot.
    fn temp_path(name: &str) -> String {
        std::env::temp_dir().join(format!("dieselrogue_test_{}.dat", name)).to_string_lossy().into_owned()
    }

    /// A small run: the player carrying a primed grenade, an alerted guard, ammo on the floor.
    fn sample_world() -> World {
        let mut world = World::new_test();
        world.create_player(Point { x: 10, y: 10 }, Direction::Up, "Player".into()).unwrap();
        let guard = world.create_light_guard(Point { x: 20, y: 10 }, Direction::Left).unwrap();
        let mut ai = ActorAI::new(Profile::Guard { anchor: Point { x: 20, y: 10 }, combat_tactic: CombatTactic::Hold });
        ai.alert = AlertLevel::Alert { last_known: Point { x: 12, y: 10 }, search_ticks: 7 };
        world.entities[guard].ai = AI::Actor(ai);

        let mut grenade = Item::grenade();
        grenade.id = 900;
        grenade.active = true;
        grenade.inventory_actions.retain(|a| a.id != ActionId::Prime);
        world.entities[0].body.inventory.push(grenade);

        let _ = world.add_item(Point { x: 5, y: 5 }, Item::ammo_bullets());
        world.player_xp = 1234;
        world
    }

    fn actions(item: &Item) -> Vec<ActionId> {
        item.inventory_actions.iter().chain(&item.equip_actions).map(|a| a.id).collect()
    }

    #[test]
    fn round_trip_preserves_the_run() {
        let path = temp_path("round_trip");
        let world = sample_world();
        let mut log = GameLog { entries: vec![] };
        log.log("one".to_string());
        log.log_enemy("two".to_string());
        save_to(&path, &world, &log, 42, 777).unwrap();
        let loaded = load_from(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!((loaded.turn, loaded.seed), (42, 777));
        assert_eq!(loaded.log.entries, log.entries);
        assert_eq!(loaded.world.player_xp, 1234);
        assert_eq!(loaded.world.entities.len(), world.entities.len());
        for (a, b) in world.entities.iter().zip(&loaded.world.entities) {
            assert_eq!((&a.name, a.position), (&b.name, b.position));
            assert_eq!(a.innate_actions.len(), b.innate_actions.len(), "innate actions of {} not restored", a.name);
            assert_eq!(a.body.inventory.len(), b.body.inventory.len());
            for (x, y) in a.body.inventory.iter().zip(&b.body.inventory) {
                assert_eq!(actions(x), actions(y), "actions of {} not restored", x.name);
            }
        }

        // The primed grenade stays primed: it can be thrown but not primed again.
        let grenade = loaded.world.entities[0].body.inventory.iter().find(|i| i.id == 900).unwrap();
        assert!(grenade.active && !actions(grenade).contains(&ActionId::Prime) && actions(grenade).contains(&ActionId::Throw));

        // AI memory survives (the search clock may have ticked while re-declaring intents).
        match &loaded.world.entities[1].ai {
            AI::Actor(ai) => assert!(matches!(ai.alert, AlertLevel::Alert { last_known: Point { x: 12, y: 10 }, search_ticks } if search_ticks >= 7)),
            _ => panic!("guard lost its AI"),
        }

        let floor = &loaded.world.map.items[loaded.world.map.xy_idx(5, 5)];
        assert!(matches!(floor.as_ref().map(|i| &i.kind), Some(ItemKind::Ammo { charges: 30, .. })));
    }

    #[test]
    fn save_file_is_not_plain_text() {
        let path = temp_path("not_plain");
        save_to(&path, &sample_world(), &GameLog { entries: vec![] }, 0, 0).unwrap();
        let bytes = fs::read(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(&bytes[0..4], MAGIC);
        let text = String::from_utf8_lossy(&bytes);
        for word in ["Player", "Pistol", "Guard", "\"entities\""] {
            assert!(!text.contains(word), "save file reveals {:?}", word);
        }
    }

    #[test]
    fn damaged_or_foreign_files_are_refused() {
        let path = temp_path("refused");
        save_to(&path, &sample_world(), &GameLog { entries: vec![] }, 0, 0).unwrap();
        let good = fs::read(&path).unwrap();

        let mut damaged = good.clone();
        let mid = damaged.len() / 2;
        damaged[mid] ^= 0xFF;
        fs::write(&path, &damaged).unwrap();
        assert!(load_from(&path).is_err(), "a damaged file must be refused");

        let mut other_version = good.clone();
        other_version[4..8].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        fs::write(&path, &other_version).unwrap();
        assert!(load_from(&path).is_err(), "another format version must be refused");

        fs::write(&path, b"{\"not\": \"a save\"}").unwrap();
        assert!(load_from(&path).is_err(), "a foreign file must be refused");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn item_registry_can_rebuild_every_item() {
        let makers = Item::makers_by_name();
        assert_eq!(makers.len(), crate::item::ALL_ITEMS.len(), "item names must be unique to be rebuilt");
        for make in crate::item::EXCEPTIONAL_ITEMS {
            assert!(makers.contains_key(&make().name), "{} is missing from ALL_ITEMS", make().name);
        }
    }
}

/// XORs `bytes` with a xorshift keystream; applying it twice restores the input.
fn scramble(bytes: &mut [u8]) {
    let mut state = SCRAMBLE_KEY;
    for chunk in bytes.chunks_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        for (b, k) in chunk.iter_mut().zip(state.to_le_bytes()) {
            *b ^= k;
        }
    }
}

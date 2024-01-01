use std::path::PathBuf;

use serde_repr::Deserialize_repr;
use serde::Deserialize;

#[derive(Deserialize_repr, Debug, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum GameStatus {
    PreSongSelect = 0,
    Playing = 2,
    SongSelect = 5,
    EditorSongSelect = 4,
    ResultsScreen = 7,
    MultiplayerLobbySelect = 11,
    MultiplayerLobby = 12,
    MultiplayerResultsScreen = 14,

    #[default]
    Unknown
}

#[derive(Deserialize_repr, Debug, Default, PartialEq, Eq)]
#[repr(i16)]
pub enum BeatmapStatus {
    #[default]
    Unknown = 0,
    Unsubmitted = 1,
    Unranked = 2,
    Unused = 3,
    Ranked = 4,
    Approved = 5,
    Qualified = 6,
    Loved = 7
}

#[repr(transparent)]
#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Mods(u32);
bitflags::bitflags! {
    impl Mods: u32 {
        const None = 0;
        const NoFail = 1;
        const Easy = 1 << 1;
        const TouchDevice = 1 << 2;
        const Hidden = 1 << 3;
        const HardRock = 1 << 4;
        const SuddenDeath = 1 << 5;
        const DoubleTime = 1 << 6;
        const Relax = 1 << 7;
        const HalfTime = 1 << 8;
        const Nightcore = 1 << 9;
        const Flashlight = 1 << 10;
        const Autoplay = 1 << 11;
        const SpunOut = 1 << 12;
        const Autopilot = 1 << 13;
        const Perfect = 1 << 14;
        const Key4 = 1 << 15;
        const Key5 = 1 << 16;
        const Key6 = 1 << 17;
        const Key7 = 1 << 18;
        const Key8 = 1 << 19;
        const KeyMod = Self::Key4.bits()
            | Self::Key5.bits()
            | Self::Key6.bits()
            | Self::Key7.bits()
            | Self::Key8.bits();
        const Fadein = 1 << 20;
        const Random = 1 << 21;
        const Cinema = 1 << 22;
        const TargetPractice = 1 << 23;
        const Key9 = 1 << 24;
        const Coop = 1 << 25;
        const Key1 = 1 << 26;
        const Key3 = 1 << 27;
        const Key2 = 1 << 28;
        const ScoreV2 = 1 << 29;
        const Mirror = 1 << 30;
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct RosuValues {
    pub skin: String,
    pub beatmap_full_path: PathBuf,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub difficulty: String,
    pub map_id: i32,
    pub mapset_id: i32,
    pub beatmap_folder: String,
    pub beatmap_id: i32,
    pub beatmap_file: String,
    pub background_file: String,
    pub background_path_full: PathBuf,
    pub playtime: i32,
    pub menu_mode: u8,
    pub status: GameStatus,
    pub stars: f64,
    pub stars_mods: f64,
    pub current_stars: f64,
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub od: f32,
    pub beatmap_status: BeatmapStatus,
    pub username: String,
    pub score: i32,
    pub hit_300: i16,
    pub hit_100: i16,
    pub hit_50: i16,
    pub hit_geki: i16,
    pub hit_katu: i16,
    pub hit_miss: i16,
    pub accuracy: f64,
    pub combo: i16,
    pub max_combo: i16,
    pub mode: u8,
    pub slider_breaks: i16,
    pub unstable_rate: f64,
    pub grade: String,
    pub current_hp: f64,
    pub current_hp_smooth: f64,
    pub bpm: f64,
    pub current_bpm: f64,
    pub kiai_now: bool,
    pub current_pp: f64,
    pub fc_pp: f64,
    pub ss_pp: f64,
    pub passed_objects: usize,
    pub menu_mods: Mods,
    pub mods: Mods,
    pub mods_str: Vec<String>,
    pub plays: i32,
    pub last_obj_time: f64,
    pub first_obj_time: f64
}

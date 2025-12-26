use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct WindowDimensions {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct WindowSettings {
    pub title: String,
    pub dimensions: WindowDimensions,
    pub theme: String,
}

#[derive(Debug, Deserialize)]
pub struct GameSettings {
    #[serde(rename = "defaultFs")]
    pub default_fs: String,
    #[serde(rename = "defaultVs")]
    pub default_vs: String,
    pub model: String,
    pub texture: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub window: WindowSettings,
    pub game: GameSettings,
}

pub fn parse_settings(path: &str) -> Settings {
    let file = File::open(path).expect("File not found - did bear steal it?");
    serde_json::from_reader(file).expect("Invalid json structure - like trying to shit in upside-down toilet")
}
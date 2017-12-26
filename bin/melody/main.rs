// TODO: Use Config file
// TODO: Iterm display for album cover? (probably not)
// TODO: ticki/termion TUI
use std::path::PathBuf;
extern crate app_dirs;
extern crate config;
extern crate melody;
use app_dirs::*;

const APP_INFO: AppInfo = AppInfo {
    name: "Melody",
    author: "Fuzen",
};

use melody::*;

fn get_settings() -> Option<std::collections::HashMap<String, String>> {
    let config_dir = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
    let mut config_file = config_dir.clone();
    config_file.push("Config.toml");
    if !&config_file.exists() {
        return None;
    }
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(
            config_file
                .to_str()
                .expect("Failed to generate config file path"),
        ))
        .unwrap()
        .merge(config::Environment::with_prefix("MELODY"))
        .expect("Failed to merge config with environment");
    settings
        .try_into()
        .expect(&format!("Failed to parse settings: {:?}", config_file))
}

fn main() {
    if let Some(settings) = get_settings() {
        if let Some(music_dir) = settings.get("music") {
            play_test(PathBuf::from(music_dir))
        } else {
            let mut config_file = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
            config_file.push("Config.toml");
            println!("No music dir in {:?}", config_file);
        }
    } else {
        println!("Failed to get settings, defaulting to ~/Music");
        match std::env::home_dir() {
            Some(mut home_dir) => {
                home_dir.push("Music");
                if home_dir.exists() && home_dir.is_dir() {
                    play_test(home_dir)
                }
            }
            None => println!("Failed to find home directory, exeting"),
        }
    }
}

fn play_test(music_dir: PathBuf) {
    let mut mp =
        MusicPlayer::new(Playlist::from_dir(music_dir).expect("Failed to make playlist from dir"));
    mp.start().expect("Failed to start music player");
    mp.lock();
    while !mp.queue().is_empty() {
        mp.play_next();
        mp.lock();
    }
}

// TODO: Fix Config file
// TODO: Iterm display for album cover? (probably not)
// TODO: Media Controls
extern crate directories;
extern crate indicatif;
extern crate melody;
#[macro_use]
extern crate human_panic;
#[macro_use]
extern crate serde_derive;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

use melody::*;

#[derive(Debug, Deserialize)]
struct Settings {
    volume: f64,
    music: String,
}

impl Settings {
    pub fn new() -> Result<Self, Errors> {
        let mut config_dir = project_dir()?.config_dir().to_owned();
        ::std::fs::create_dir_all(&config_dir).is_ok(); // Result doesnt matter
        config_dir.push("Config.toml");
        if !config_dir.exists() {
            ::std::fs::File::create(&config_dir).map_err(|_| Errors::FailedToGetConfig)?;
        }
        let mut conf = config::Config::default();
        conf.set_default("volume", 0.25f64).is_ok();
        let user_dir = directories::UserDirs::new().ok_or(Errors::FailedToGetUserDir)?;
        let audio_dir = user_dir.audio_dir().ok_or(Errors::FailedToGetAudioDir)?;
        let audio_dir = if audio_dir.exists() {
            audio_dir.to_owned()
        } else {
            ::std::env::current_dir().map_err(|_| Errors::FailedToGetAudioDir)?
        };
        let audio_dir = audio_dir.to_str().ok_or(Errors::FailedToGetAudioDir)?;
        conf.set_default("music", audio_dir).is_ok();
        conf.merge(config::File::from_str(
            config_dir.to_str().ok_or(Errors::FailedToGetConfig)?,
            config::FileFormat::Toml,
        ))
        .is_ok();
        conf.merge(config::Environment::with_prefix("MELODY"))
            .is_ok();
        conf.try_into().map_err(|_| Errors::FailedToGetConfig)
    }
    pub fn volume(&self) -> f32 {
        self.volume as f32
    }
    pub fn music_dir(&self) -> PathBuf {
        PathBuf::from(&self.music)
    }
}

#[allow(unused)]
fn project_dir() -> Result<ProjectDirs, Errors> {
    ProjectDirs::from("info", "Fuzen", "Melody").ok_or(Errors::FailedToGetAppDirectory)
}

#[derive(Debug)]
enum Errors {
    FailedToGetAppDirectory,
    FailedToGetUserDir,
    FailedToGetAudioDir,
    FailedToGetConfig,
    FailedToCreatePlaylist,
}

fn generate_progress_bar(s: Song) -> ProgressBar {
    let pb = ProgressBar::new(s.duration.as_secs());
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] ({eta_precise})",
            )
            .progress_chars("#>-"),
    );
    pb.set_message(&format!(
        "{} - {} - {}",
        s.artist.unwrap_or(String::from("Unknown Artist")),
        s.album.unwrap_or(String::from("Unkwon Album")),
        s.title.unwrap_or(String::from("Unkown Title"))
    ));
    pb
}

fn main() {
    // setup_panic!();
    play_test().unwrap()
}

fn play_test() -> Result<(), Errors> {
    let config = Settings::new()?;
    println!("Getting playlist from settings: {:#?}", config);
    let playlist = Playlist::from_dir(config.music_dir()).ok_or(Errors::FailedToCreatePlaylist)?;
    let mut mp = MusicPlayer::new(playlist);
    println!("Getting volume");
    mp.set_volume(config.volume());
    drop(config);
    mp.shuffle();
    println!("{}", mp);
    mp.start().expect("Failed to start music player");
    let mut pb = match mp.status() {
        MusicPlayerStatus::NowPlaying(song) => generate_progress_bar(song),
        _ => unreachable!(),
    };
    loop {
        match mp.status() {
            MusicPlayerStatus::NowPlaying(song) => {
                pb.set_position(song.elapsed.as_secs());
            }
            MusicPlayerStatus::Stopped(_) => {
                if mp.queue().is_empty() {
                    break;
                } else {
                    mp.play_next();
                    pb = match mp.status() {
                        MusicPlayerStatus::NowPlaying(song) => generate_progress_bar(song),
                        _ => unreachable!(),
                    };
                }
            }
            _ => (),
        }
        thread::sleep(Duration::from_millis(250))
    }
    println!("{}", mp.status());
    println!("End of playlist, goodbye!");
    Ok(())
}

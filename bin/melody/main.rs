// TODO: Fix Config file
// TODO: Iterm display for album cover? (probably not)
// TODO: Media Controls
extern crate config;
extern crate directories;
extern crate indicatif;
extern crate melody;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;

use melody::*;

fn project_dir() -> Result<ProjectDirs, Errors> {
    ProjectDirs::from("info", "Fuzen", "Melody").ok_or(Errors::FailedToGetAppDirectory)
}

#[derive(Debug)]
enum Errors {
    NoConfig,
    FailedToGetAppDirectory,
    FailedToGetUserDir,
    FailedToGetAudioDir,
    FailedConfigMergeWithEnvironment,
    FailedToParseSettings,
}
fn generate_progress_bar(s: Song) -> ProgressBar {
    let pb = ProgressBar::new(s.duration.as_secs());
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} {msg} [{elapsed_precise}] [{bar:40.cyan/blue}] ({eta_precise})",
            ).progress_chars("#>-"),
    );
    pb.set_message(&format!(
        "{} - {} - {}",
        s.artist.unwrap_or(String::from("Unknown Artist")),
        s.album.unwrap_or(String::from("Unkwon Album")),
        s.title.unwrap_or(String::from("Unkown Title"))
    ));
    pb
}

fn get_settings() -> Result<std::collections::HashMap<String, String>, Errors> {
    let project_dir = project_dir()?;
    let config_dir = project_dir.config_dir().to_owned();

    let mut config_file = config_dir.clone();
    config_file.push("Config.toml");
    if !&config_file.exists() {
        return Err(Errors::NoConfig);
    }
    config::Config::default()
        .merge(config::File::with_name(
            config_file
                .to_str()
                .ok_or(Errors::FailedToGetAppDirectory)?,
        )).map_err(|_| Errors::FailedToParseSettings)
        .and_then(|settings| {
            settings
                .merge(config::Environment::with_prefix("MELODY_"))
                .map_err(|_| Errors::FailedConfigMergeWithEnvironment)
        }).and_then(|s| {
            s.clone()
                .try_into::<HashMap<String, String>>()
                .map_err(|_| Errors::FailedToParseSettings)
        })
}

fn main() {
    println!("{:?}", project_dir().unwrap().config_dir());
    if let Ok(settings) = get_settings() {
        if let Some(music_dir) = settings.get("MUSIC") {
            play_test(PathBuf::from(music_dir))
        } else {
            let mut config_file = project_dir().unwrap().config_dir().to_owned();
            config_file.push("Config.toml");
            println!("No music dir in {:?}", config_file);
        }
    } else {
        let music_dir = ::std::env::var("MELODY_MUSIC")
            .and_then(|v| Ok(PathBuf::from(v)))
            .unwrap_or(
                directories::UserDirs::new()
                    .ok_or(Errors::FailedToGetUserDir)
                    .and_then(|user_dir| {
                        user_dir
                            .audio_dir()
                            .ok_or(Errors::FailedToGetAudioDir)
                            .and_then(|audio_dir| Ok(audio_dir.to_owned()))
                    }).unwrap(),
            );
        println!(
            "Failed to get settings, defaulting to {}",
            music_dir.to_str().unwrap()
        );
        if !music_dir.exists() {
            println!("Directory does not exist, exiting...");
            return ();
        }
        play_test(music_dir)
    }
}

fn play_test(music_dir: PathBuf) {
    let mut mp =
        MusicPlayer::new(Playlist::from_dir(music_dir).expect("Failed to make playlist from dir"));
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
}

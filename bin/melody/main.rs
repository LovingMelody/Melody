// TODO: Fix Config file
// TODO: Iterm display for album cover? (probably not)
// TODO: Media Controls
extern crate directories;
extern crate indicatif;
extern crate melody;
#[macro_use]
extern crate human_panic;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};

use melody::*;

#[allow(unused)]
fn project_dir() -> Result<ProjectDirs, Errors> {
    ProjectDirs::from("info", "Fuzen", "Melody").ok_or(Errors::FailedToGetAppDirectory)
}

#[derive(Debug)]
enum Errors {
    FailedToGetAppDirectory,
    FailedToGetUserDir,
    FailedToGetAudioDir,
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

fn main() {
    setup_panic!();
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
            "Looking for music in {}",
            music_dir.to_str().unwrap_or("your os's default music dir")
        );
        if !music_dir.exists() {
            println!("Directory does not exist, exiting...");
            return ();
        }
        play_test(music_dir)
}

fn play_test(music_dir: PathBuf) {
    let playlist = match Playlist::from_dir(music_dir.clone()) {
        None => {
            println!("Failed to create playlist from {}. Exiting...", music_dir.to_str().unwrap_or("Music dir"));
            return;
        },
        Some(pl) => pl
    };
    drop(music_dir);
    let mut mp = MusicPlayer::new(playlist);
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

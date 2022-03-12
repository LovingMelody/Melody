#![warn(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
// TODO: Fix Config file
// TODO: Iterm display for album cover? (probably not)
// TODO: Media Controls
extern crate directories;
extern crate indicatif;
extern crate melody;
#[macro_use]
extern crate human_panic;
use std::thread;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use melody::*;

mod config;
use config::Settings;

#[derive(Debug)]
pub enum Errors {
    FailedToGetAppDirectory,
    FailedToGetUserDir,
    FailedToGetAudioDir,
    FailedToGetConfig,
    FailedToCreatePlaylist,
    FailedToStartMusicPlayer,
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
    let msg = format!(
        "{} - {} - {}",
        s.artist.unwrap_or_else(|| String::from("Unknown Artist")),
        s.album.unwrap_or_else(|| String::from("Unkwon Album")),
        s.title.unwrap_or_else(|| String::from("Unkown Title"))
    );
    pb.set_message(msg);
    pb
}

fn main() {
    setup_panic!();
    play_test().unwrap()
}

fn play_test() -> Result<(), Errors> {
    let config = Settings::new()?;
    let playlist = if config.prioritize_cwd {
        ::std::env::current_dir()
            .ok()
            .and_then(Playlist::from_dir)
            .and_then(|pl| if pl.is_empty() { None } else { Some(pl) })
            .or_else(|| Playlist::from_dir(config.music.clone()))
            .ok_or(Errors::FailedToCreatePlaylist)
    } else {
        Playlist::from_dir(config.music.clone())
            .and_then(|pl| if pl.is_empty() { None } else { Some(pl) })
            .or_else(|| ::std::env::current_dir().ok().and_then(Playlist::from_dir))
            .ok_or(Errors::FailedToCreatePlaylist)
    }?;
    let mut mp = MusicPlayer::new(playlist);
    mp.set_volume(config.volume);
    drop(config);
    mp.shuffle();
    mp.start().unwrap();
    println!("{}", mp);
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

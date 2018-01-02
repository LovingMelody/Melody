use utils::{fmt_duration, supported_song};
use song::{Playlist, Song};
use std::fmt;
use rodio;
use errors::{MelodyErrors, MelodyErrorsKind};
use std::fs::File;
use tabwriter::TabWriter;
use std::io::{BufReader, Write};

#[derive(Clone)]
pub enum MusicPlayerStatus {
    Stopped(Option<Song>),
    NowPlaying(Song),
    Paused(Song),
}

impl fmt::Display for MusicPlayerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MusicPlayerStatus::*;
        match self.clone() {
            Paused(song) => write!(f, "[Paused] : {}", song),
            NowPlaying(song) => write!(f, "[Now Playing] : {}", song),
            Stopped(s) => match s {
                Some(song) => write!(f, "[Stopped] : Last Played - {}", song),
                None => write!(f, "[Stopped]"),
            },
        }
    }
}

pub struct MusicPlayer {
    playlist: Vec<Song>,
    sink: rodio::Sink,
    current: Option<Song>,
    previous: Option<Song>,
}

impl MusicPlayer {
    pub fn new(playlist: Playlist) -> Self {
        // TODO: USe a non-depricated method
        #[allow(deprecated)]
        let endpoint =
            rodio::get_default_endpoint().expect("Failed to find default music endpoint");

        MusicPlayer {
            // Remove all unsuported songs
            playlist: c![song, for song in playlist.tracks, if supported_song(song.file())],
            sink: rodio::Sink::new(&endpoint),
            current: None,
            previous: None,
        }
    }
    pub fn start(&mut self) -> Result<(), MelodyErrors> {
        if self.playlist.is_empty() {
            Err(MelodyErrors::new(
                MelodyErrorsKind::EmptyQueue,
                "Playlist is empty",
                None,
            ))
        } else {
            if self.sink.empty() {
                let current = self.playlist.remove(0);
                let file =
                    File::open(&current).expect(&format!("Failed to read {:?}", current.file));
                let source = rodio::Decoder::new(BufReader::new(file))
                    .expect(&format!("Failed to decode {:?}", current.file));
                self.sink.append(source);
                self.current = Some(current);
            };
            Ok(())
        }
    }
    pub fn resume(&self) {
        self.sink.play();
    }
    pub fn pause(&self) {
        self.sink.pause();
    }
    pub fn stop(&mut self) {
        self.sink.stop();
        self.previous = self.current.clone();
        self.current = None;
    }
    pub fn play_next(&mut self) {
        self.stop();
        self.start().unwrap_or(());
    }
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume)
    }
    pub fn lock(&self) {
        self.sink.sleep_until_end();
    }
    pub fn queue(&self) -> &Vec<Song> {
        &self.playlist
    }
    pub fn status(&self) -> MusicPlayerStatus {
        if let Some(song) = self.current.clone() {
            if self.sink.is_paused() {
                MusicPlayerStatus::Paused(song)
            } else {
                MusicPlayerStatus::NowPlaying(song)
            }
        } else {
            MusicPlayerStatus::Stopped(self.previous.clone())
        }
    }
}

impl fmt::Display for MusicPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let now_playing = match self.current {
            Some(ref song) => {
                let status: &str = if self.sink.is_paused() {
                    "[paused] "
                } else {
                    ""
                };
                format!("{}{}\n", status, song)
            }
            None => String::new(),
        };
        let mut tw = TabWriter::new(Vec::new());
        let mut lines: Vec<String> = vec![
            String::from("|\tArtist\t|\tAlbum\t|\tTitle\t|\tDuration\t|"),
        ];
        for track in &self.playlist {
            let duration = match track.duration {
                Some(ref time) => fmt_duration(time),
                None => String::from("?"),
            };
            lines.push(format!(
                "|\t{}\t|\t{}\t|\t{}\t|\t{}\t|",
                track.artist().unwrap_or("Unkown Artist"),
                track.album().unwrap_or("Unkown Album"),
                track.title().unwrap_or("Unkown Title"),
                duration
            ))
        }
        write!(tw, "{}", lines.join("\n")).unwrap();
        tw.flush().unwrap();
        write!(
            f,
            "{}{}",
            now_playing,
            String::from_utf8(tw.into_inner().unwrap()).unwrap()
        )
    }
}

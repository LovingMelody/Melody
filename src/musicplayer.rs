use errors::{MelodyErrors, MelodyErrorsKind};
use rand::{thread_rng, Rng};
use rodio;
use song::{Playlist, Song};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Write};
use tabwriter::TabWriter;
use utils::{fmt_duration, supported_song};

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
    playing_time: (::std::time::Instant, ::std::time::Duration),
}

impl MusicPlayer {
    pub fn new(playlist: Playlist) -> Self {
        // TODO: USe a non-depricated method
        #[allow(deprecated)]
        let endpoint =
            rodio::default_output_device().expect("Failed to find default music endpoint");

        MusicPlayer {
            // Remove all unsuported songs
            playlist: c![song, for song in playlist.tracks, if supported_song(song.file())],
            sink: rodio::Sink::new(&endpoint),
            current: None,
            previous: None,
            playing_time: (
                ::std::time::Instant::now(),
                ::std::time::Duration::from_secs(0),
            ),
        }
    }

    pub fn shuffle(&mut self) {
        thread_rng().shuffle(&mut self.playlist);
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
                self.playing_time.0 = ::std::time::Instant::now();
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
    pub fn resume(&mut self) {
        self.sink.play();
        self.playing_time.0 = ::std::time::Instant::now();
    }
    pub fn pause(&mut self) {
        self.sink.pause();
        self.playing_time.1 = self.playing_time.0.elapsed();
    }
    pub fn stop(&mut self) {
        self.sink.stop();
        self.previous = self.current.clone().and_then(|mut s| {
            s.elapsed = self.playing_time.0.elapsed() + self.playing_time.1;
            Some(s)
        });
        self.current = None;
    }
    pub fn play_next(&mut self) {
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
        if self.sink.empty() {
            MusicPlayerStatus::Stopped(self.previous.clone())
        } else if let Some(mut song) = self.current.clone() {
            if self.sink.is_paused() {
                song.elapsed = self.playing_time.1;
                MusicPlayerStatus::Paused(song)
            } else {
                song.elapsed = self.playing_time.0.elapsed() + self.playing_time.1;
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
        let mut lines: Vec<String> = vec![String::from(
            "|\tArtist\t|\tAlbum\t|\tTitle\t|\tDuration\t|",
        )];
        for track in &self.playlist {
            let duration = fmt_duration(&track.duration);
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

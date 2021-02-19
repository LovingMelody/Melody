use errors::{MelodyErrors, MelodyErrorsKind};
use rand::{seq::SliceRandom, thread_rng};
use rodio;
use song::{Playlist, Song};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Write};
use tabwriter::TabWriter;
use utils::fmt_duration;

/// Music Player Status
/// Showing the status of the Music player
#[derive(Clone, Debug)]
pub enum MusicPlayerStatus {
    /// Music player has stopped
    /// Contains the previus song if any
    Stopped(Option<Song>),
    /// Now playing: song
    NowPlaying(Song),
    /// Paused: Song
    Paused(Song),
}

/// Displays the following
/// [Paused] : {Song} @ Time stamp
/// [Now Playing] : Song
/// [Stopped] : Last Played - Song
/// [Stopped]
impl fmt::Display for MusicPlayerStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MusicPlayerStatus::*;
        match self.clone() {
            Paused(song) => write!(f, "[Paused] : {} @ {}", song, fmt_duration(&song.elapsed)),
            NowPlaying(song) => write!(f, "[Now Playing] : {}", song),
            Stopped(s) => match s {
                Some(song) => write!(f, "[Stopped] : Last Played - {}", song),
                None => write!(f, "[Stopped]"),
            },
        }
    }
}
// TODO: Implement Back
/// Music Player
pub struct MusicPlayer {
    // Stream must remain for audio to play
    #[allow(dead_code)]
    stream: rodio::OutputStream,
    /// Songs in Queue
    playlist: Vec<Song>,
    /// Audio controller
    sink: rodio::Sink,
    /// Current song
    current: Option<Song>,
    /// Previus song
    previous: Option<Song>,
    /// Used to find the current the song's play time
    /// `::std::time::instant` is used for start time
    /// `::std::time::Duration` is used for storing postion in the event of pause
    playing_time: (::std::time::Instant, ::std::time::Duration),
}

impl MusicPlayer {
    /// Cronstructs a new MusicPlayer
    pub fn new(playlist: Playlist) -> Self {
        // Audio endpoint (EX: Alsa)
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to find default music endpoint");

        MusicPlayer {
            stream,
            // Remove all unsuported songs
            playlist: playlist.tracks, //c![song, for song in playlist.tracks, if supported_song(song.file())],
            // Create audio controller
            sink: rodio::Sink::try_new(&stream_handle).unwrap(),
            current: None,
            previous: None,
            playing_time: (
                ::std::time::Instant::now(),
                ::std::time::Duration::from_secs(0),
            ),
        }
    }

    /// Shuffle the order of the playlist
    pub fn shuffle(&mut self) {
        self.playlist.shuffle(&mut thread_rng());
    }

    /// Plays the first song in the Queue if any
    /// Otherwise throws an error
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
                // TODO: Make this return an error
                let file = File::open(&current)
                    .unwrap_or_else(|_| panic!("Failed to read {:#?}", current.file));
                // TODO: Make this return an error
                let source = rodio::Decoder::new(BufReader::new(file))
                    .unwrap_or_else(|_| panic!("Failed to decode {:#?}", current.file));
                self.sink.append(source);

                self.current = Some(current);
            };
            Ok(())
        }
    }

    /// Resume's the song
    /// Should only be used of the song was paused
    /// Or it messes with the song's progress counter
    // TODO: Fix error when called when not stopped
    pub fn resume(&mut self) {
        self.sink.play();
        self.playing_time.0 = ::std::time::Instant::now();
    }
    /// Pauses Song
    pub fn pause(&mut self) {
        self.sink.pause();
        // Update Song's playing time
        self.playing_time.1 += self.playing_time.0.elapsed();
    }

    /// Stop's currently playing song
    // TODO: Fix error if no current song
    pub fn stop(&mut self) {
        self.sink.stop();
        self.previous = self.current.clone().map(|mut s| {
            s.elapsed = self.playing_time.0.elapsed() + self.playing_time.1;
            s
        });
        self.current = None;
    }
    /// Play next Song in Queue
    /// TODO: Return something if there is nothing else
    pub fn play_next(&mut self) {
        self.start().unwrap_or(());
    }
    /// Returns the music players volume
    /// Volume precentage is represented as a decimal
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    /// Set the volume of the music player
    /// volume: Precentage as a decimal
    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume)
    }
    /// Lock current thread until current song ends
    pub fn lock(&self) {
        self.sink.sleep_until_end();
    }
    /// List current songs in queue
    pub fn queue(&self) -> &Vec<Song> {
        &self.playlist
    }
    /// Return the music players status
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

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        self.sink.stop()
    }
}

use crate::utils::{fmt_duration, list_files, supported_song};
use std::convert::AsRef;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::errors::MelodyErrors;
use lofty::Accessor;
use tabwriter::TabWriter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Song {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<u32>,
    pub genre: Option<String>,
    pub duration: Duration,
    pub file: PathBuf,
    pub elapsed: Duration,
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = fmt_duration(&self.duration);
        write!(
            f,
            "{} - {} - {} ({})",
            self.artist().unwrap_or("Unkown Artist"),
            self.album().unwrap_or("Unkown Album"),
            self.title().unwrap_or("Unkown Title"),
            duration
        )
    }
}

impl Song {
    /// Optionally return the artist of the song
    /// If `None` it wasnt able to read the tags
    pub fn artist(&self) -> Option<&str> {
        match self.artist.as_ref() {
            Some(artist) => Some(artist),
            None => None,
        }
    }
    /// Optionally return the song's album
    /// If `None` failed to read the tags
    pub fn album(&self) -> Option<&str> {
        match self.album.as_ref() {
            Some(album) => Some(album),
            None => None,
        }
    }
    /// Optionally return the title of the song
    /// If `None` it wasnt able to read the tags
    pub fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title),
            None => None,
        }
    }
    /// Optionally returns the song's track number
    /// If `None` it wasnt able to read the tags
    pub fn track(&self) -> Option<u32> {
        self.track
    }
    /// Optionally returns the song's genere
    /// If `None` it wasnt able to read the tags
    pub fn genre(&self) -> Option<&str> {
        match self.genre.as_ref() {
            Some(genre) => Some(genre),
            None => None,
        }
    }
    /// Returns the `Duration` of the song
    pub fn duration(&self) -> Duration {
        self.duration
    }
    /// Returns the elapsed time the song has been played
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    /// Returns the path of the song
    pub fn file(&self) -> &Path {
        &self.file
    }
    /// Load song from Pathbuf
    pub fn load(file: PathBuf) -> Result<Self, MelodyErrors> {
        let track: Option<u32> = None;
        let tagged_file = lofty::Probe::open(&file).map_err(|e| {
            MelodyErrors::new(e.into(), "Invalid path provided", Some(file.as_path()))
        })?.read(true)
        .map_err(|e| MelodyErrors::new(e.into(), "Failed to read file", Some(file.as_path())))?;
        let metadata = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or_else(|| MelodyErrors::new(crate::MelodyErrorsKind::FailedToReadTag, "Failed to get file tags", Some(file.as_path())))?
        };
        Ok(Self {
            artist: metadata.artist().map(|s| s.to_owned()),
            album: metadata.album().map(|s| s.to_owned()),
            title: metadata.title().map(|s| s.to_owned()),
            genre: metadata.genre().map(|s| s.to_owned()),
            track,
            duration: tagged_file.properties().duration(),
            file,
            elapsed: Duration::from_millis(0),
        })
    }
    /// Checks if the song is the same
    /// if matching_genre is true it will check genre as well
    pub fn matching_song(&self, s: &Song, matching_genre: bool) -> bool {
        if self.artist() != s.artist() {
            return false;
        }
        if self.album() != s.album() {
            return false;
        }
        if self.title() != s.title() {
            return false;
        }
        if self.track() != s.track() {
            return false;
        }
        if (self.genre() != s.genre()) && matching_genre {
            return false;
        }
        if self.track() != s.track() {
            return false;
        }
        if self.duration() == s.duration() {
            return false;
        }
        true
    }
    /// Checks if the song is an exact match
    /// Checks the song's tags and if the path is the same
    pub fn exact_match(&self, s: &Song, same_path: bool) -> bool {
        self.matching_song(s, true) && ((self.file() == s.file()) | same_path)
    }
}

impl AsRef<Path> for Song {
    fn as_ref(&self) -> &Path {
        &self.file
    }
}

/// Collection of Songs
pub struct Playlist {
    pub tracks: Vec<Song>,
}

impl Playlist {
    /// Create a playlist from a directory
    /// will walk through the directory and
    /// collect the songs it can process
    pub fn from_dir(path: PathBuf) -> Option<Self> {
        if !path.exists() {
            return None;
        };
        if path.is_file() {
            if let Ok(song) = Song::load(path) {
                return Some(Self { tracks: vec![song] });
            } else {
                return None;
            }
        };
        println!("Collecting tracks..");
        let mut tracks: Vec<Song> = list_files(path)
            .filter_map(|f| {
                if supported_song(&f) {
                    Song::load(f).ok()
                } else {
                    None
                }
            })
            .collect();
        tracks.dedup();
        Some(Self { tracks })
    }
    /// Returns if the playlist is currently empty
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

impl From<Vec<Song>> for Playlist {
    fn from(tracks: Vec<Song>) -> Self {
        Self { tracks }
    }
}

impl fmt::Display for Playlist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tw = TabWriter::new(Vec::new());
        let mut lines: Vec<String> = vec![String::from(
            "|\tArtist\t|\tAlbum\t|\tTitle\t|\tDuration\t|",
        )];
        for track in &self.tracks {
            lines.push(format!(
                "|\t{}\t|\t{}\t|\t{}\t|\t{}\t|",
                track.artist().unwrap_or("Unkown Artist"),
                track.album().unwrap_or("Unkown Album"),
                track.title().unwrap_or("Unkown Title"),
                fmt_duration(&track.duration)
            ))
        }
        write!(tw, "{}", lines.join("\n")).unwrap();
        tw.flush().unwrap();
        f.write_str(&String::from_utf8(tw.into_inner().unwrap()).unwrap())
    }
}

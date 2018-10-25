use std::convert::AsRef;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use utils::{fmt_duration, genre_to_string, list_files, supported_song};

use errors::MelodyErrors;
use tabwriter::TabWriter;

#[derive(Clone, Eq, PartialEq)]
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
        let duration = format!("{}", fmt_duration(&self.duration));
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
    pub fn artist(&self) -> Option<&str> {
        match self.artist.as_ref() {
            Some(artist) => Some(artist),
            None => None,
        }
    }
    pub fn album(&self) -> Option<&str> {
        match self.album.as_ref() {
            Some(album) => Some(album),
            None => None,
        }
    }
    pub fn title(&self) -> Option<&str> {
        match self.title.as_ref() {
            Some(title) => Some(title),
            None => None,
        }
    }
    pub fn track(&self) -> Option<u32> {
        self.track
    }
    pub fn genre(&self) -> Option<&str> {
        match self.genre.as_ref() {
            Some(genre) => Some(genre),
            None => None,
        }
    }
    pub fn duration(&self) -> Duration {
        self.duration
    }
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    pub fn file(&self) -> &Path {
        &self.file
    }
    pub fn load(file: PathBuf) -> Result<Self, MelodyErrors> {
        use mp3_metadata;

        let mut artist: Option<String> = None;
        let mut album: Option<String> = None;
        let mut title: Option<String> = None;
        let mut genre: Option<String> = None;
        let track: Option<u32> = None;
        let metadata = match mp3_metadata::read_from_file(&file) {
            Err(e) => Err(MelodyErrors::new(
                e.into(),
                "Failed to read meta data",
                Some(&file),
            ))?,
            Ok(m) => m,
        };
        let duration = metadata.duration;
        if let Some(tag) = metadata.tag {
            if !tag.title.is_empty() {
                title = Some(tag.title)
            }
            if !tag.artist.is_empty() {
                artist = Some(tag.artist)
            }
            if !tag.album.is_empty() {
                album = Some(tag.album)
            }
            genre = Some(genre_to_string(&tag.genre))
        }
        Ok(Self {
            artist,
            album,
            title,
            genre,
            track,
            duration,
            file,
            elapsed: Duration::from_millis(0),
        })
    }
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
    pub fn exact_match(&self, s: &Song) -> bool {
        self.matching_song(s, true) && self.file() == s.file()
    }
}

impl AsRef<Path> for Song {
    fn as_ref(&self) -> &Path {
        &self.file
    }
}

pub struct Playlist {
    pub tracks: Vec<Song>,
}

impl Playlist {
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
        // let tracks = c![Song::load(file), for file in list_files(path), if supported_song(&file)];
        let files = c![f, for f in list_files(path), if supported_song(&f)];
        let mut tracks: Vec<Song> = (c![Song::load(file), for file in files])
            .into_iter()
            // Filter the errors and songs without duration
            .filter_map(|s| s.ok())
            .collect();
        tracks.dedup();
        Some(Self { tracks })
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

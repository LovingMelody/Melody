use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fmt;
use std::convert::AsRef;
use std::io::Write;
use utils::{fmt_duration, genre_to_string, list_files, supported_song};

use tabwriter::TabWriter;
use errors::{MelodyErrors, MelodyErrorsKind};
use std::error::Error as StdError;

#[derive(Clone)]
pub struct Song {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<u32>,
    pub genre: Option<String>,
    pub duration: Option<Duration>,
    pub file: PathBuf,
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = match self.duration {
            Some(ref time) => format!("{}", fmt_duration(time)),
            None => String::from("Unkown Duration"),
        };
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
    pub fn duration(&self) -> Option<Duration> {
        self.duration
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
        match mp3_metadata::read_from_file(&file) {
            Ok(metadata) => {
                let duration = Some(metadata.duration);
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
                })
            }
            Err(_) => {
                // TODO: Put this in a error log
                // println!("{:?}",
                //     MelodyErrors::new(MelodyErrorsKind::MetaDataError(err), err.description(), Some(&file)));
                Ok(Self {
                    artist,
                    album,
                    title,
                    genre,
                    track,
                    duration: None,
                    file,
                })
            }
        }
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
        let tracks = c![Song::load(file), for file in files];
        Some(Self {
            tracks: c![s.expect("Error loading dir"), for s in tracks, if s.is_ok()],
        })
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
        let mut lines: Vec<String> = vec![
            String::from("|\tArtist\t|\tAlbum\t|\tTitle\t|\tDuration\t|"),
        ];
        for track in &self.tracks {
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
        f.write_str(&String::from_utf8(tw.into_inner().unwrap()).unwrap())
    }
}

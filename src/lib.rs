// TODO: Implement tag edit feature
// TODO: Auto Tag Detect (duration etc)
// TODO: Write tests
// TODO: Use a config file ()
// TODO: Music Player

extern crate rtag;
extern crate tabwriter;
use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::fmt;
use tabwriter::TabWriter;
use std::io::{BufReader, Write};
#[macro_use]
extern crate cute;
extern crate rodio;

/// Add to library
/// `from` - Original Directory that the music being moved from, must be an absolute path
/// `to` - New Directory that the music is being moved to, must be an absolute path
/// `from` must be in a folder above or on the same level as ``to``
/// Files that failed to be sorted will remain
#[derive(Debug)]
pub struct MelodyErrors {
    kind: MelodyErrorsKind,
    description: String,
    file: Option<PathBuf>,
}

impl StdError for MelodyErrors {
    fn description(&self) -> &str {
        &self.description
    }
}
impl fmt::Display for MelodyErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl MelodyErrors {
    pub fn new(kind: MelodyErrorsKind, description: &str, file: Option<&Path>) -> Self {
        Self {
            kind,
            description: description.to_string(),
            file: match file {
                Some(file) => Some(file.to_path_buf()),
                None => None,
            },
        }
    }
    pub fn kind(&self) -> MelodyErrorsKind {
        self.kind
    }
    pub fn file(&self) -> Option<PathBuf> {
        self.file.clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MelodyErrorsKind {
    Io(IoErrorKind),
    NotAbsolutePath,
    PathDoesNotExist,
    PathIsNotADir,
    PathIsNotAFile,
    ChildOfParentRecursion,
    FailedToFindParent,
    UnsupportedFileType,
    FailedToReadTag,
    UnkownFileType,
    CanNotReadFileEXT,
    EmptyQueue,
    NotPaused,
    AlreadyPlaying,
}

fn fmt_duration(time: &std::time::Duration) -> String {
    let mut sec: u64 = time.as_secs();
    let hour: u64 = sec / 3600;
    let min = sec % 3600;
    sec = min % 60;
    let mut time = String::new();
    if hour != 0 {
        time.push_str(&format!("{}h ", hour));
    };
    if min != 0 {
        time.push_str(&format!("{}m ", min));
    }
    if sec != 0 {
        time.push_str(&format!("{}s ", sec));
    }
    time.trim().to_string()
}

#[derive(Clone)]
pub struct Song {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<u32>,
    pub genre: Option<String>,
    pub duration: Option<std::time::Duration>,
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
        match self.artist {
            Some(ref artist) => Some(&artist),
            None => None,
        }
    }
    pub fn album(&self) -> Option<&str> {
        match self.album {
            Some(ref album) => Some(&album),
            None => None,
        }
    }
    pub fn title(&self) -> Option<&str> {
        match self.title {
            Some(ref title) => Some(&title),
            None => None,
        }
    }
    pub fn track(&self) -> Option<u32> {
        self.track
    }
    pub fn genre(&self) -> Option<&str> {
        match self.genre {
            Some(ref genre) => Some(&genre),
            None => None,
        }
    }
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.duration
    }
    pub fn file(&self) -> &Path {
        &self.file
    }
    pub fn load(file: PathBuf) -> Option<Self> {
        use rtag::frame::*;
        use rtag::metadata::Unit;
        use rtag::metadata::MetadataReader;
        use rodio::source::Source;
        let mut artist: Option<String> = None;
        let mut album: Option<String> = None;
        let mut title: Option<String> = None;
        let mut genre: Option<String> = None;
        let mut track: Option<u32> = None;
        if let Ok(meta) = MetadataReader::new(&file.to_str()?) {
            for m in meta {
                match m {
                    Unit::FrameV1(frame) => {
                        if !frame.artist.is_empty() {
                            artist = Some(frame.artist)
                        }
                        if !frame.album.is_empty() {
                            album = Some(frame.album)
                        }
                        if !frame.title.is_empty() {
                            title = Some(frame.title)
                        }
                        if !frame.genre.is_empty() {
                            genre = Some(frame.genre)
                        }
                        if !frame.track.is_empty() {
                            match frame.track.parse() {
                                Ok(num) => {
                                    track = Some(num);
                                }
                                Err(_) => (),
                            }
                        }
                    }
                    Unit::FrameV2(_, ref frame) => {
                        let _ = frame.to_map();
                        if let Ok(frame) = frame.to_map() {
                            if let Some(value) = frame.get("artist") {
                                artist = Some(value.clone())
                            }
                            if let Some(value) = frame.get("album") {
                                album = Some(value.clone())
                            }
                            if let Some(value) = frame.get("title") {
                                title = Some(value.clone())
                            }
                            if let Some(value) = frame.get("genre") {
                                genre = Some(value.clone())
                            }
                            if let Some(value) = frame.get("tracl") {
                                if let Ok(num) = value.parse() {
                                    track = Some(num)
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        } else {
            return None;
        }
        let duration = match File::open(&file) {
            Err(_) => {
                return None;
            }
            Ok(f) => match rodio::Decoder::new(BufReader::new(f)) {
                Ok(source) => source.total_duration(),
                Err(_) => {
                    return None;
                }
            },
        };
        Some(Self {
            artist,
            album,
            title,
            genre,
            track,
            duration,
            file,
        })
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
            return Some(Self {
                tracks: vec![Song::load(path)?],
            });
        };
        let tracks = c![Song::load(file), for file in list_files(path), if supported_song(&file)];
        Some(Self {
            tracks: c![s.expect("Error loading dir"), for s in tracks, if s.is_some()],
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
                track.track.unwrap_or(0),
                duration
            ))
        }
        write!(tw, "{}", lines.join("\n")).unwrap();
        tw.flush().unwrap();
        f.write_str(&String::from_utf8(tw.into_inner().unwrap()).unwrap())
    }
}

impl From<IoError> for MelodyErrors {
    fn from(err: IoError) -> Self {
        MelodyErrors::new(MelodyErrorsKind::Io(err.kind()), err.description(), None)
    }
}

/// Recursively list all the files of Path, if path is a file, return vec![path]
fn list_files(path: PathBuf) -> Vec<PathBuf> {
    if path.is_file() {
        vec![path; 1]
    } else {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut folders: Vec<PathBuf> = vec![path];
        while !folders.is_empty() {
            let folder = folders.pop().expect("There is no folders");
            if let Ok(entries) = fs::read_dir(folder) {
                for entry in entries {
                    if let Ok(e) = entry {
                        let p = e.path();
                        if p.is_file() {
                            files.push(p)
                        } else {
                            folders.push(p)
                        }
                    }
                }
            }
        }
        files
    }
}

fn get_filetype(path: &Path) -> Option<String> {
    Some(path.extension()?.to_str()?.to_lowercase())
}

fn supported_song(path: &Path) -> bool {
    path.exists() && path.is_file() && match get_filetype(path) {
        Some(ext) => match ext.as_str() {
            "flac" | "wav" | "vorbis" => true,
            _ => false,
        },
        None => false,
    }
}
fn organize_song(song: Song, mut to: PathBuf) -> Result<(), MelodyErrors> {
    use self::MelodyErrorsKind::*;
    if song.file().is_dir() {
        return Err(MelodyErrors::new(
            PathIsNotAFile,
            "Song is not a file",
            Some(song.file()),
        ));
    }
    match song.artist() {
        Some(artist) => to.push(artist),
        None => to.push("Uknown Artist"),
    }
    match song.album() {
        Some(album) => to.push(album),
        None => to.push("Unkown Album"),
    }
    fs::create_dir_all(&to)?;
    to.push(song.file().file_name().expect("Failed to read file name"));
    fs::copy(song, to)?;
    Ok(())
}

pub fn add_to_library(from: &Path, to: &Path) -> Result<Option<Vec<MelodyErrors>>, MelodyErrors> {
    use self::MelodyErrorsKind::*;
    if from.is_relative() {
        return Err(MelodyErrors::new(
            NotAbsolutePath,
            "`from` path is not absolute",
            None,
        ));
    };
    if to.is_relative() {
        return Err(MelodyErrors::new(
            NotAbsolutePath,
            "`to` path is not absolute",
            None,
        ));
    };
    if !from.exists() {
        return Err(MelodyErrors::new(
            Io(IoErrorKind::NotFound),
            "`from` path is not absolute",
            None,
        ));
    };
    if !to.exists() {
        return Err(MelodyErrors::new(
            Io(IoErrorKind::NotFound),
            "`to` path is not absolute",
            None,
        ));
    };
    if from.is_file() {
        return Err(MelodyErrors::new(
            PathIsNotADir,
            "`from` is not a directory",
            None,
        ));
    };
    if to.is_file() {
        return Err(MelodyErrors::new(
            PathIsNotADir,
            "`to` is not a directory",
            None,
        ));
    };
    match from.parent() {
        Some(parent) => if parent == to {
            return Err(MelodyErrors::new(
                ChildOfParentRecursion,
                "`from` is a direc child of to",
                None,
            ));
        },
        None => {
            return Err(MelodyErrors::new(
                FailedToFindParent,
                "could not find parent of `from`.",
                None,
            ))
        }
    };
    let files = list_files(from.to_path_buf());
    let mut skipped: Vec<MelodyErrors> = Vec::new();
    for file in &files {
        match Song::load(file.to_path_buf()) {
            Some(song) => match organize_song(song, to.to_path_buf()) {
                Ok(_) => (),
                Err(e) => skipped.push(e),
            },
            None => skipped.push(MelodyErrors::new(
                UnsupportedFileType,
                "Unsupported file type",
                Some(file),
            )),
        }
    }
    if skipped.is_empty() {
        Ok(None)
    } else {
        Ok(Some(skipped))
    }
}

impl std::convert::AsRef<std::path::Path> for Song {
    fn as_ref(&self) -> &Path {
        &self.file
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
        return &self.playlist;
    }
}

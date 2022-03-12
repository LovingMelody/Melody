use crate::errors::{MelodyErrors, MelodyErrorsKind};
use crate::song::{Playlist, Song};
use num_integer::div_mod_floor;
use std::fs;
use std::io::ErrorKind as IoErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

/// Helper function to format `Duration` to a string
pub fn fmt_duration(time: &Duration) -> String {
    let (min, sec) = div_mod_floor(time.as_secs(), 60);
    let (hour, min) = div_mod_floor(min, 60);
    let mut time_str = String::new();
    if hour != 0 {
        time_str.push_str(&format!("{}h ", hour));
    };
    if min != 0 {
        time_str.push_str(&format!("{}m ", min));
    }
    if sec != 0 {
        time_str.push_str(&format!("{}s ", sec));
    }
    time_str.trim().to_string()
}

/// Ignored file filter, if a file starts with `.#` its ignored
pub fn ignored_file(p: &Path) -> bool {
    match p.file_name() {
        Some(file_name) => match file_name.to_str() {
            Some(name) => !name.starts_with(".#"),
            None => false,
        },
        None => false,
    }
}

/// Recursively list all the files of Path (if not ignored), if path is a file, return vec![path]
pub fn list_files(path: PathBuf) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok().map(|e| e.into_path()))
        .filter(|p| ignored_file(p))
}
/// Optionally returns the file's extention as a String
pub fn get_filetype(path: &Path) -> Option<String> {
    Some(path.extension()?.to_str()?.to_lowercase())
}

/// Checks if a song is supported
/// If a song's file extention is
/// flac | wav | vorbis | mp3 | ogg
/// its supported
pub fn supported_song(path: &Path) -> bool {
    path.exists()
        && path.is_file()
        && match get_filetype(path) {
            Some(ext) => matches!(ext.as_str(), "flac" | "wav" | "vorbis" | "mp3" | "ogg"),
            None => false,
        }
}

/// Function meant to organize a specific song
/// Song: Song to organize
/// to: Directory move song to
pub fn organize_song(song: Song, mut to: PathBuf) -> Result<(), MelodyErrors> {
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

/// Add to library
/// `from` - Original Directory that the music being moved from, must be an absolute path
/// `to` - New Directory that the music is being moved to, must be an absolute path
/// `from` must be in a folder above or on the same level as ``to``
/// Files that failed to be sorted will remain
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
        Some(parent) => {
            if parent == to {
                return Err(MelodyErrors::new(
                    ChildOfParentRecursion,
                    "`from` is a direc child of to",
                    None,
                ));
            }
        }
        None => {
            return Err(MelodyErrors::new(
                FailedToFindParent,
                "could not find parent of `from`.",
                None,
            ))
        }
    };
    let files = list_files(from.to_path_buf());
    Ok(Some(
        files
            .filter_map(|f| match Song::load(f) {
                Ok(_) => None,
                Err(e) => Some(e),
            })
            .collect(),
    ))
}


/// Find  Duplicates
/// `music_dir` - Music directory to find duplicates
/// Returns a list of duplicates
pub fn find_duplicates(music_dir: &Path) -> Vec<PathBuf> {
    fn list_occ(s: &Song, v: &[Song]) -> Vec<usize> {
        let mut occ = vec![];
        for (pos, song) in v.iter().enumerate() {
            if song.matching_song(s, false) {
                occ.push(pos)
            }
        }
        occ
    }
    let music_dir = music_dir.to_path_buf();
    if let Some(pl) = Playlist::from_dir(music_dir) {
        let tracks = pl.tracks;
        let mut dupes = Vec::with_capacity(tracks.len());
        // TODO: convert it into a lazy iter
        for track in &tracks {
            let occ = list_occ(track, &tracks);
            if occ.len() == 1 {
                continue;
            }
            for pos in occ {
                if !dupes.contains(&pos) {
                    dupes.push(pos)
                }
            }
        }
        dupes.shrink_to_fit();
        dupes.iter().map(|pos| tracks[*pos].file.clone()).collect()
    } else {
        Vec::with_capacity(0)
    }
}

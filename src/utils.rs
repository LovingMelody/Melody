use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fs;
use errors::{MelodyErrors, MelodyErrorsKind};
use song::Song;
use std::io::ErrorKind as IoErrorKind;
use mp3_metadata::Genre;
use num_integer::div_mod_floor;
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

/// Recursively list all the files of Path, if path is a file, return vec![path]
pub fn list_files(path: PathBuf) -> Vec<PathBuf> {
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

pub fn get_filetype(path: &Path) -> Option<String> {
    Some(path.extension()?.to_str()?.to_lowercase())
}

pub fn supported_song(path: &Path) -> bool {
    path.exists() && path.is_file() && match get_filetype(path) {
        Some(ext) => match ext.as_str() {
            "flac" | "wav" | "vorbis" => true,
            _ => false,
        },
        None => false,
    }
}

/// Add to library
/// `from` - Original Directory that the music being moved from, must be an absolute path
/// `to` - New Directory that the music is being moved to, must be an absolute path
/// `from` must be in a folder above or on the same level as ``to``
/// Files that failed to be sorted will remain
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
            Ok(song) => match organize_song(song, to.to_path_buf()) {
                Ok(_) => (),
                Err(e) => skipped.push(e),
            },
            Err(e) => skipped.push(e),
        }
    }
    if skipped.is_empty() {
        Ok(None)
    } else {
        Ok(Some(skipped))
    }
}

pub fn genre_to_string(from: &Genre) -> String {
    use self::Genre::*;
    match from.clone() {
        Something(txt) => txt,
        _ => format!("{:?}", from),
    }
}

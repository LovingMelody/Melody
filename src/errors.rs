use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::{Path, PathBuf};

#[derive(Debug)]
/// Errors from Melody
pub struct MelodyErrors {
    /// Kind of error
    kind: MelodyErrorsKind,
    /// Description of error
    description: String,
    /// Path to file related to error
    file: Option<PathBuf>,
}

impl StdError for MelodyErrors {
    fn description(&self) -> &str {
        &self.description
    }
}
impl fmt::Display for MelodyErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description)
    }
}
/// MelodyErrors
/// # Example
/// ```
/// match ::std::env::current_dir() {
///     Ok(path) => {
///         if !path.exists() {
///             Err(melody::MelodyErrors::new(melody::MelodyErrorsKind::PathDoesNotExist, "Path does not exist", Some(&path)))
///         } else {
///             Ok(path)
///         }
///     },
///     Err(e) => Err(e.into())
/// };
/// ```
impl MelodyErrors {
    pub fn new(kind: MelodyErrorsKind, description: &str, file: Option<&Path>) -> Self {
        Self {
            kind,
            description: description.to_string(),
            file: file.map(|f| f.to_path_buf()),
        }
    }
    pub fn kind(&self) -> &MelodyErrorsKind {
        &self.kind
    }
    pub fn file(&self) -> Option<PathBuf> {
        self.file.clone()
    }
}

impl From<IoError> for MelodyErrors {
    fn from(err: IoError) -> Self {
        MelodyErrors::new(MelodyErrorsKind::Io(err.kind()), &format!("{}", err), None)
    }
}

// impl From<Mp3MetadataError> for MelodyErrors {
//     fn from(err: Mp3MetadataError) -> Self {
// 	       MelodyErrors {
//		       kind: MelodyErrorsKind::Mp3MetadataError(err),
// 			   description: err.description().to_string(),
// 			   file: None
// 		  }
//     }
// }

/// Kind of errror that arose from Melody
#[derive(Debug)]
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
    MissingDuration,
    MetaDataError(lofty::error::LoftyError),
}
impl ::std::convert::From<lofty::error::LoftyError> for MelodyErrorsKind {
    fn from(e: lofty::error::LoftyError) -> MelodyErrorsKind {
        MelodyErrorsKind::MetaDataError(e)
    }
}

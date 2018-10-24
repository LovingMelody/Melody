use mp3_metadata::Error as Mp3MetadataError;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::{Path, PathBuf};
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

impl From<IoError> for MelodyErrors {
    fn from(err: IoError) -> Self {
        MelodyErrors::new(MelodyErrorsKind::Io(err.kind()), err.description(), None)
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
    MissingDuration,
    MetaDataError(Mp3MetadataError),
}
impl ::std::convert::From<Mp3MetadataError> for MelodyErrorsKind {
    fn from(e: Mp3MetadataError) -> MelodyErrorsKind {
        MelodyErrorsKind::MetaDataError(e)
    }
}

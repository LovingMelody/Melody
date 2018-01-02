// TODO: Implement tag edit feature
// TODO: Auto Tag Detect (duration etc)
// TODO: Write tests
// TODO: Use a config file ()
// TODO: Music Player
extern crate mp3_metadata;
extern crate num_integer;
extern crate tabwriter;

#[macro_use]
extern crate cute;
extern crate rodio;

mod errors;
mod song;
mod utils;

mod musicplayer;

pub use musicplayer::{MusicPlayer, MusicPlayerStatus};
pub use song::{Playlist, Song};
pub use errors::{MelodyErrors, MelodyErrorsKind};
pub use utils::add_to_library;

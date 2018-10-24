// TODO: Implement tag edit feature
// TODO: Stable Tag Detection (duration etc) currently iffy
// TODO: Write tests

extern crate mp3_metadata;
extern crate num_integer;
extern crate rayon;
extern crate tabwriter;
#[macro_use]
extern crate cute;
extern crate rand;
extern crate rodio;

mod errors;
mod song;
mod utils;

mod musicplayer;

pub use errors::{MelodyErrors, MelodyErrorsKind};
pub use musicplayer::{MusicPlayer, MusicPlayerStatus};
pub use song::{Playlist, Song};
pub use utils::{add_to_library, find_duplicates, fmt_duration};

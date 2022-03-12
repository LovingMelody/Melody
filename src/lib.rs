// TODO: Implement tag edit feature
// TODO: Stable Tag Detection (duration etc) currently iffy
// TODO: Write tests
// TODO: Write benchmarks
// TODO: Write docs

// extern crate num_integer;
// extern crate rand;
// extern crate rodio;
// extern crate tabwriter;
// extern crate walkdir;

mod errors;
mod song;
mod utils;

mod musicplayer;

pub use errors::{MelodyErrors, MelodyErrorsKind};
pub use musicplayer::{MusicPlayer, MusicPlayerStatus};
pub use song::{Playlist, Song};
pub use utils::{add_to_library, find_duplicates, fmt_duration, organize_song};

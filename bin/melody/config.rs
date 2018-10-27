use directories::{ProjectDirs, UserDirs};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use Errors;

fn project_dir() -> Result<ProjectDirs, Errors> {
    ProjectDirs::from("info", "Fuzen", "Melody").ok_or(Errors::FailedToGetAppDirectory)
}

#[derive(Debug)]
pub struct Settings {
    pub volume: f32,
    pub music: PathBuf,
    pub prioritize_cwd: bool,
}

impl Settings {
    pub fn new() -> Result<Settings, Errors> {
        let project_dir = project_dir()?;
        let mut config_file = project_dir.config_dir().to_owned();
        ::std::fs::create_dir_all(&config_file).is_ok(); // Result doesnt matter
        config_file.push("Config.melody");
        println!("Config is located at: {:#?}", config_file);
        let mut settings = if !config_file.exists() {
            Self::create_default(config_file)?
        } else {
            let file = ::std::fs::File::open(config_file).map_err(|_| Errors::FailedToGetConfig)?;
            let mut volume: f32 = 0.25;
            let mut prioritize_cwd: bool = false;
            let mut music: Option<PathBuf> = None;
            for line in BufReader::new(file).lines() {
                if let Ok(line) = line {
                    if line.starts_with("volume=") {
                        if let Some(v) = line.get(7..) {
                            if let Ok(v) = v.parse() {
                                volume = v;
                            }
                        }
                    } else if line.starts_with("music=") {
                        if let Some(v) = line.get(6..) {
                            let p = PathBuf::from(v);
                            if p.exists() {
                                music = Some(p)
                            }
                        }
                    } else if line.starts_with("prioritize_cwd") {
                        if let Some(v) = line.get(15..) {
                            if let Ok(v) = v.parse() {
                                prioritize_cwd = v;
                            }
                        }
                    }
                }
            }
            let music = match music {
                Some(m) => m,
                None => {
                    let user_dir = UserDirs::new().ok_or(Errors::FailedToGetUserDir)?;
                    user_dir
                        .audio_dir()
                        .ok_or(Errors::FailedToGetAudioDir)?
                        .to_owned()
                }
            };
            Self {
                volume,
                prioritize_cwd,
                music,
            }
        };
        if let Ok(v) = ::std::env::var("MELODY_VOLUME") {
            if let Ok(v) = v.parse() {
                settings.volume = v;
            }
        }
        if let Ok(v) = ::std::env::var("MELODY_MUSIC") {
            let p = PathBuf::from(v);
            if p.exists() {
                settings.music = p;
            }
        }
        if let Ok(v) = ::std::env::var("MELODY_PRIORITIZE_CWD") {
            if let Ok(v) = v.parse() {
                settings.prioritize_cwd = v;
            }
        }
        Ok(settings)
    }

    fn create_default(config_file: PathBuf) -> Result<Self, Errors> {
        let user_dir = UserDirs::new().ok_or(Errors::FailedToGetUserDir)?;
        let audio_dir = user_dir
            .audio_dir()
            .ok_or(Errors::FailedToGetAudioDir)?
            .to_owned();
        let mut f = ::std::fs::File::create(config_file).map_err(|_| Errors::FailedToGetConfig)?;
        let file_txt = format!(
            "volume={}\nmusic={}\nprioritize_cwd={}",
            0.25,
            audio_dir.to_str().ok_or(Errors::FailedToGetAudioDir)?,
            false
        );
        f.write_all(file_txt.as_bytes())
            .map_err(|_| Errors::FailedToGetConfig)?;
        Ok(Settings {
            volume: 0.25,
            music: audio_dir,
            prioritize_cwd: false,
        })
    }
}

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::ffi::OsStr;
use std::fmt;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum WallpaperError {
    CommandError(Error),
    DirectoryNotFound,
    ImageNotFound,
    InvalidPath,
}

impl fmt::Display for WallpaperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::CommandError(err) => format!("error trying to set GNOME setting: {}", err),
            Self::DirectoryNotFound => String::from("directory not found"),
            Self::ImageNotFound => String::from("unable to find a JPEG or PNG"),
            Self::InvalidPath => String::from("does not appear to be valid path"),
        };
        write!(f, "{}", message)
    }
}

const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

fn is_image_ext(ext: &OsStr) -> bool {
    ext.to_str()
        .is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

fn gsettings_set(schema: &str, key: &str, file_name: &str) -> Result<(), WallpaperError> {
    Command::new("gsettings")
        .arg("set")
        .arg(schema)
        .arg(key)
        .arg(format!("file://{}", file_name))
        .output()
        .map_err(WallpaperError::CommandError)?;
    Ok(())
}

fn select_wallpaper(wallpaper_dir: &Path) -> Result<String, WallpaperError> {
    // select all PNG and JPEG files in directory
    let paths: Vec<PathBuf> = wallpaper_dir
        .read_dir()
        .map_err(|_| WallpaperError::DirectoryNotFound)?
        .flatten()
        .map(|e| e.path())
        .filter(|path| path.extension().is_some_and(is_image_ext))
        .collect();

    // choose one path at random
    let file_name = paths
        .choose(&mut thread_rng())
        .ok_or(WallpaperError::ImageNotFound)?
        .to_str()
        .ok_or(WallpaperError::InvalidPath)?;

    Ok(file_name.to_string())
}

pub fn change_wallpaper(wallpaper_dir: &Path) -> Result<String, WallpaperError> {
    // select a random wallpaper path and apply Gnome desktop settings
    let file_name = select_wallpaper(wallpaper_dir)?;

    for (schema, key) in [
        ("org.gnome.desktop.background", "picture-uri"),
        ("org.gnome.desktop.background", "picture-uri-dark"),
        ("org.gnome.desktop.screensaver", "picture-uri"),
    ] {
        gsettings_set(schema, key, &file_name)?;
    }
    Ok(file_name)
}

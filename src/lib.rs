use rand::seq::SliceRandom;
use rand::thread_rng;
use std::ffi::{OsStr, OsString};
use std::fs::DirEntry;
use std::io::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum WallpaperError {
    CommandError(Error),
    DirectoryNotFound,
    ImageNotFound,
}

const IMAGE_EXTENSIONS: [&str; 2] = ["jpg", "png"];

fn is_image_ext(ext: &OsStr) -> bool {
    IMAGE_EXTENSIONS
        .iter()
        .map(OsString::from)
        .collect::<Vec<OsString>>()
        .contains(&ext.to_ascii_lowercase())
}

fn matches_image_path(result: Result<DirEntry, Error>) -> Option<PathBuf> {
    match result {
        Ok(entry) => match entry.path() {
            path => match path.extension() {
                Some(ext) if is_image_ext(ext) => Some(path),
                _ => None,
            },
        },
        _ => None,
    }
}

fn gsettings_set(schema: &str, key: &str, file_name: &str) -> Result<(), WallpaperError> {
    match Command::new("gsettings")
        .arg("set")
        .arg(schema)
        .arg(key)
        .arg(format!("file://{}", file_name))
        .output()
    {
        Err(err) => Err(WallpaperError::CommandError(err)),
        _ => Ok(()),
    }
}

pub fn select_wallpaper(wallpaper_dir: &PathBuf) -> Result<PathBuf, WallpaperError> {
    if let Ok(wallpaper_dir) = wallpaper_dir.read_dir() {
        let paths: Vec<PathBuf> = wallpaper_dir.filter_map(matches_image_path).collect();
        match paths.choose(&mut thread_rng()) {
            Some(path) => Ok(path.clone()),
            _ => Err(WallpaperError::ImageNotFound),
        }
    } else {
        Err(WallpaperError::DirectoryNotFound)
    }
}

pub fn change_wallpaper(path: &PathBuf) -> Result<&PathBuf, WallpaperError> {
    if let Some(file_name) = path.to_str() {
        for (schema, key) in vec![
            ("org.gnome.desktop.background", "picture-uri"),
            ("org.gnome.desktop.background", "picture-uri-dark"),
            ("org.gnome.desktop.screensaver", "picture-uri"),
        ] {
            gsettings_set(schema, key, file_name)?
        }
        Ok(path)
    } else {
        Err(WallpaperError::ImageNotFound)
    }
}

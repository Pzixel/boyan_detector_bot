use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SETTINGS_FILE : &str = "Settings.toml";
const TARGET_FILE : &str = "boyan_detector_bot.exe";

fn main() {
    let current_dir = env::current_dir().unwrap();
    if let Ok(Some(path)) = get_exe_dir(&current_dir) {
        let dest_path = Path::new(&path).join(SETTINGS_FILE);
        fs::copy(SETTINGS_FILE, dest_path).unwrap();
    }
}

fn get_exe_dir(dir: &PathBuf) -> io::Result<Option<PathBuf>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(inner_result) = get_exe_dir(&path)?{
                    return Ok(Some(inner_result));
                }
            } else {
                if entry.file_name() == std::ffi::OsStr::new(TARGET_FILE){
                    return Ok(Some(dir.to_path_buf()));
                }
            }
        }
    }
    Ok(None)
}
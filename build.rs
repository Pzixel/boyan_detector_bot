use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SETTINGS_FILE : &str = "Settings.toml";
const LOG4RS_FILE : &str = "log4rs.toml";
const TARGET_FILE : &str = "boyan_detector_bot.exe";

fn main() {
    let current_dir = env::current_dir().unwrap();
    if let Ok(Some(target_dir_path)) = get_exe_dir(&current_dir) {
        copy(&target_dir_path, SETTINGS_FILE);
        copy(&target_dir_path, LOG4RS_FILE);
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

fn copy<S: AsRef<std::ffi::OsStr> + ?Sized, P: Copy + AsRef<Path>>(target_dir_path: &S, file_name: P) {
    fs::copy(file_name, Path::new(&target_dir_path).join(file_name)).unwrap();
}
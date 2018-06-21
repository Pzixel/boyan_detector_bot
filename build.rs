use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const LOG4RS_FILE: &str = "log4rs.toml";

fn main() {
    let target_dir_path = env::var("OUT_DIR").unwrap();
    copy(&target_dir_path, LOG4RS_FILE);
}

fn copy<S: AsRef<std::ffi::OsStr> + ?Sized, P: Copy + AsRef<Path>>(target_dir_path: &S, file_name: P) {
    fs::copy(file_name, Path::new(&target_dir_path).join(file_name)).unwrap();
}

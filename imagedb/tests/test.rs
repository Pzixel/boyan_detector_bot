extern crate imagedb;
extern crate cv;

use std::path::{Path, PathBuf};
use std::fs;

#[test]
fn it_works() {
    let file = fs::read(get_asset_path("lenna.png")).unwrap();
    let mut db = imagedb::InMemoryDatabase::new();
    db.insert(&file);
    assert_eq!(2 + 2, 4);
}

pub fn get_asset_path(name: &'static str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("assets").join(name)
}
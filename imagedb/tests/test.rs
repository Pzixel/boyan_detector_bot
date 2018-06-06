extern crate imagedb;
extern crate cv;

use std::path::{Path, PathBuf};
use std::fs;
use imagedb::Database;

#[test]
fn it_works() {
    let lenna = fs::read(get_asset_path("lenna.png")).unwrap();
    let lenna_demotivator = fs::read(get_asset_path("lenna_demotivator.png")).unwrap();
    let db = imagedb::InMemoryDatabase::new();
    let mut storage = imagedb::Storage::new(db);
    let result = storage.save_image_if_new(&lenna);
    let result_demotivator = storage.save_image_if_new(&lenna_demotivator);
    assert_eq!(result, imagedb::ImageVariant::New);
    assert_eq!(result_demotivator, imagedb::ImageVariant::AlreadyExists);
}

pub fn get_asset_path(name: &'static str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("assets").join(name)
}
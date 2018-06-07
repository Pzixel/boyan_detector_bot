extern crate imagedb;
extern crate cv;

use std::path::{Path, PathBuf};
use std::fs;

#[test]
fn it_works() {
    let lenna = fs::read(get_asset_path("lenna.png")).unwrap();
    let lenna_demotivator = fs::read(get_asset_path("lenna_demotivator.png")).unwrap();
    let solvay_conference = fs::read(get_asset_path("Solvay_conference_1927.jpg")).unwrap();
    let db = imagedb::InMemoryDatabase::new();
    let mut storage = imagedb::Storage::new(db);
    let result = storage.save_image_if_new(&lenna, "lenna");
    let result_demotivator = storage.save_image_if_new(&lenna_demotivator, "lenna demotivator");
    let result_solvay_conference = storage.save_image_if_new(&solvay_conference, "solvay_conference");
    assert_eq!(result, imagedb::ImageVariant::New);
    assert_ne!(result_demotivator, imagedb::ImageVariant::New);
    assert_eq!(result_solvay_conference, imagedb::ImageVariant::New);
}

pub fn get_asset_path(name: &'static str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("assets").join(name)
}
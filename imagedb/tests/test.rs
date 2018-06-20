extern crate cv;
extern crate imagedb;

use imagedb::*;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn it_works() {
    let lenna = fs::read(get_asset_path("lenna.png")).unwrap();
    let lenna_demotivator = fs::read(get_asset_path("lenna_demotivator.png")).unwrap();
    let solvay_conference = fs::read(get_asset_path("Solvay_conference_1927.jpg")).unwrap();
    let db = imagedb::InMemoryDatabase::new();
    let mut storage = imagedb::Storage::new(db);
    let result = storage.save_image_if_new(Image::new(lenna, "1".into(), "lenna"));
    let result_demotivator = storage.save_image_if_new(Image::new(lenna_demotivator, "2".into(), "lenna demotivator"));
    let result_solvay_conference =
        storage.save_image_if_new(Image::new(solvay_conference, "3".into(), "solvay_conference"));
    assert!(result.is_new());
    assert!(!result_demotivator.is_new());
    assert!(result_solvay_conference.is_new());
}

pub fn get_asset_path(name: &'static str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(name)
}

extern crate cv;
extern crate imagedb;

use imagedb::*;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn detects_similar_images() {
    let lenna = fs::read(get_asset_path("lenna.png")).unwrap();
    let lenna_demotivator = fs::read(get_asset_path("lenna_demotivator.png")).unwrap();
    let solvay_conference = fs::read(get_asset_path("Solvay_conference_1927.jpg")).unwrap();
    let storage = imagedb::InMemoryStorage::new();
    let mut db = imagedb::ImageDb::new(storage);
    let result = db.save_image_if_new(Image::new(lenna, "1".into(), "lenna"));
    let result_demotivator = db.save_image_if_new(Image::new(lenna_demotivator, "2".into(), "lenna demotivator"));
    let result_solvay_conference = db.save_image_if_new(Image::new(solvay_conference, "3".into(), "solvay_conference"));
    assert_eq!(result, ImageVariant::New);
    assert_eq!(result_demotivator, ImageVariant::AlreadyExists("lenna"));
    assert_eq!(result_solvay_conference, ImageVariant::New);
}

#[test]
fn could_load_images() {
    let lenna = fs::read(get_asset_path("lenna.png")).unwrap();
    let lenna_demotivator = fs::read(get_asset_path("lenna_demotivator.png")).unwrap();
    let solvay_conference = fs::read(get_asset_path("Solvay_conference_1927.jpg")).unwrap();
    let lenna = Image::new(lenna, "1".into(), "lenna");
    let lenna_demotivator = Image::new(lenna_demotivator, "2".into(), "lenna demotivator");
    let solvay_conference = Image::new(solvay_conference, "3".into(), "solvay_conference");

    let mut storage = imagedb::InMemoryStorage::new();
    storage.save_image(&lenna);
    storage.save_image(&solvay_conference);

    let mut db = imagedb::ImageDb::new(storage);

    let result = db.save_image_if_new(lenna);
    let result_demotivator = db.save_image_if_new(lenna_demotivator);
    let result_solvay_conference = db.save_image_if_new(solvay_conference);

    assert_eq!(2, db.image_count());
    assert_eq!(result, ImageVariant::AlreadyExists("lenna"));
    assert_eq!(result_demotivator, ImageVariant::AlreadyExists("lenna"));
    assert_eq!(
        result_solvay_conference,
        ImageVariant::AlreadyExists("solvay_conference")
    );
}

pub fn get_asset_path(name: &'static str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(name)
}

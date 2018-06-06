extern crate cv;

use cv::*;
use cv::imgcodecs::*;
use cv::phash::*;

pub trait Database {
    fn save_image(&mut self, image: &[u8]);
    fn load_images(&self) -> Vec<Vec<u8>>;
}

pub struct InMemoryDatabase {
    images: Vec<Vec<u8>>
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            images: Vec::new()
        }
    }
}

impl Database for InMemoryDatabase {
    fn save_image(&mut self, image: &[u8]) {
        self.images.push(image.into());
    }

    fn load_images(&self) -> Vec<Vec<u8>> {
        self.images.clone()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ImageVariant {
    New,
    AlreadyExists
}

pub struct Storage<T: Database> {
    database: T,
    hasher: PHash,
    images: Vec<Mat>
}

impl<T: Database> Storage<T> {
    pub fn new(database: T) -> Self {
        Self {
            database,
            hasher: PHash::new(),
            images: Vec::new()
        }
    }
}

impl<T: Database> Storage<T> {
    pub fn save_image_if_new(&mut self, image: &[u8]) -> ImageVariant {
        const DIFF: f64 = 0.5;

        let mat = Mat::image_decode(image, ImageReadMode::Grayscale);
        let result = ImageVariant::New;
        let mut last_diff = std::f64::INFINITY;
        for image in self.images.iter() {
            let diff = self.hasher.compare(&mat, &image);
            if diff < last_diff {
                last_diff = diff;
            }
        }
        if last_diff < DIFF {
            return ImageVariant::AlreadyExists;
        }
        self.database.save_image(image);
        self.images.push(mat);
        ImageVariant::New
    }
}
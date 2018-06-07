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
pub enum ImageVariant<D: Clone> {
    New,
    AlreadyExists(D)
}

pub struct Storage<T: Database, D: Clone> {
    database: T,
    hasher: PHash,
    images: Vec<(Mat, D)>
}

impl<T: Database, D: Clone> Storage<T, D> {
    pub fn new(database: T) -> Self {
        Self {
            database,
            hasher: PHash::new(),
            images: Vec::new()
        }
    }
}

impl<T: Database, D: Clone> Storage<T, D> {
    pub fn save_image_if_new(&mut self, image: &[u8], data: D) -> ImageVariant<D> {
        const DIFF: f64 = 0.5;

        let mat = Mat::image_decode(image, ImageReadMode::Grayscale);
        let mat = self.hasher.compute(&mat);
        let mut last_diff = std::f64::INFINITY;
        let mut result: Option<D> = None;
        for &(ref image, ref d) in self.images.iter() {
            let diff = self.hasher.compare(&mat, &image);
            if diff < last_diff {
                last_diff = diff;
                result = Some(d.clone());
            }
        }
        if last_diff < DIFF {
            return ImageVariant::AlreadyExists(result.unwrap());
        }
        self.database.save_image(image);
        self.images.push((mat, data));
        ImageVariant::New
    }
}
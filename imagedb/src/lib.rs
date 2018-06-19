#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate cv;

use cv::*;
use cv::imgcodecs::*;
use cv::hash::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image<T: Clone> {
    bytes: Vec<u8>,
    metadata: T
}

impl<T: Clone> Image<T> {
    pub fn new(bytes: Vec<u8>, medatata: T) -> Self {
        Self {
            bytes: bytes,
            metadata: medatata
        }
    }
}

pub trait Database<T: Clone> {
    fn save_image(&mut self, image: &Image<T>);
    fn load_images(&self) -> Vec<Image<T>>;
}

pub struct InMemoryDatabase<T: Clone> {
    images: Vec<Image<T>>
}

impl<T: Clone> InMemoryDatabase<T> {
    pub fn new() -> Self {
        Self {
            images: Vec::new()
        }
    }
}

impl<T: Clone> Database<T> for InMemoryDatabase<T> {
    fn save_image(&mut self, image: &Image<T>) {
        self.images.push(image.clone());
    }

    fn load_images(&self) -> Vec<Image<T>> {
        self.images.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ImageVariant<T: Clone> {
    New,
    AlreadyExists(T)
}

impl<T: Clone> ImageVariant<T> {
    pub fn is_new(&self) -> bool {
        match self {
            ImageVariant::New => true,
            _ => false
        }
    }
}

pub struct Storage<T: Clone, D: Database<T>> {
    database: D,
    hasher: ColorMomentHash,
    images: Vec<(Mat, Image<T>)>
}

impl<T: Clone, D: Database<T>> Storage<T, D> {
    pub fn new(database: D) -> Self {
        Self {
            database,
            hasher: ColorMomentHash::new(),
            images: Vec::new()
        }
    }
}

impl<T: Clone, D: Database<T>> Storage<T, D> {
    pub fn save_image_if_new(&mut self, image: Image<T>) -> ImageVariant<Image<T>> {
        const DIFF: f64 = 1.0;

        let mat = Mat::image_decode(&image.bytes, ImageReadMode::Color);
        let mat = self.hasher.compute(&mat);
        let mut last_diff = std::f64::INFINITY;
        let mut result: Option<Image<T>> = None;
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
        self.database.save_image(&image);
        self.images.push((mat, image));
        ImageVariant::New
    }
}
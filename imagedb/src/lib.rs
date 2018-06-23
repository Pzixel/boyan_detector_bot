#[macro_use]
extern crate serde_derive;

extern crate cv;
extern crate serde;
extern crate serde_json;

use cv::hash::*;
use cv::imgcodecs::*;
use cv::*;
use std::cmp::PartialEq;
use std::fs;
use std::fs::File;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image<T: Clone> {
    bytes: Vec<u8>,
    file_id: String,
    metadata: T,
}

impl<T: Clone> Image<T> {
    pub fn new(bytes: Vec<u8>, file_id: String, metadata: T) -> Self {
        Self {
            bytes,
            file_id,
            metadata,
        }
    }
}

pub trait Database<T: Clone> {
    fn save_image(&mut self, image: &Image<T>);
    fn load_images(&self) -> Vec<Image<T>>;
}

pub struct InMemoryDatabase<T: Clone> {
    images: Vec<Image<T>>,
}

impl<T: Clone> InMemoryDatabase<T> {
    pub fn new() -> Self {
        Self { images: Vec::new() }
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
    AlreadyExists(T),
}

impl<T: Clone + PartialEq> PartialEq for ImageVariant<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageVariant::New, ImageVariant::New) => true,
            (ImageVariant::AlreadyExists(a), ImageVariant::AlreadyExists(b)) if a == b => true,
            _ => false,
        }
    }
}

pub struct Storage<T: Clone, D: Database<T>> {
    database: D,
    hasher: ColorMomentHash,
    images: Vec<(Mat, Image<T>)>,
}

impl<T: Clone, D: Database<T>> Storage<T, D> {
    pub fn new(database: D) -> Self {
        Self {
            database,
            hasher: ColorMomentHash::new(),
            images: Vec::new(),
        }
    }
}

impl<T: Clone, D: Database<T>> Storage<T, D> {
    pub fn save_image_if_new(&mut self, image: Image<T>) -> ImageVariant<T> {
        const DIFF: f64 = 1.0;

        let mat = Mat::image_decode(&image.bytes, ImageReadMode::Color);
        let mat = self.hasher.compute(&mat);
        let mut last_diff = std::f64::INFINITY;
        let mut result: Option<T> = None;
        for &(ref image, ref d) in self.images.iter() {
            let diff = self.hasher.compare(&mat, &image);
            if diff < last_diff {
                last_diff = diff;
                result = Some(d.metadata.clone());
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

pub struct FileStorage<T> {
    path: PathBuf,
    marker_: PhantomData<T>,
}

impl<T> FileStorage<T> {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path,
            marker_: PhantomData,
        }
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> Database<T> for FileStorage<T> {
    fn save_image(&mut self, image: &Image<T>) {
        let path = self.path.join(&image.file_id);
        let file = File::create(path).unwrap();
        serde_json::to_writer(file, image).unwrap();
    }

    fn load_images(&self) -> Vec<Image<T>> {
        let entries = fs::read_dir(&self.path).unwrap();
        entries
            .map(|entry| {
                let reader = File::open(entry.unwrap().path()).unwrap();
                let result: Image<T> = serde_json::from_reader(reader).unwrap();
                result
            })
            .collect()
    }
}

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

pub trait Metatada: Clone {
    fn file_id(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image<T: Metatada> {
    pub bytes: Vec<u8>,
    pub metadata: T,
}

impl<T: Metatada> Image<T> {
    pub fn new(bytes: Vec<u8>, metadata: T) -> Self {
        Self { bytes, metadata }
    }
}

pub trait Storage<T: Metatada> {
    fn save_image(&mut self, image: &Image<T>);
    fn load_images(&self) -> Vec<Image<T>>;
}

pub struct InMemoryStorage<T: Metatada> {
    images: Vec<Image<T>>,
}

impl<T: Metatada> InMemoryStorage<T> {
    pub fn new() -> Self {
        Self { images: Vec::new() }
    }
}

impl<T: Metatada> Storage<T> for InMemoryStorage<T> {
    fn save_image(&mut self, image: &Image<T>) {
        self.images.push(image.clone());
    }

    fn load_images(&self) -> Vec<Image<T>> {
        self.images.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ImageVariant<T: Metatada> {
    New,
    AlreadyExists(T),
}

impl<T: Metatada + PartialEq> PartialEq for ImageVariant<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageVariant::New, ImageVariant::New) => true,
            (ImageVariant::AlreadyExists(a), ImageVariant::AlreadyExists(b)) if a == b => true,
            _ => false,
        }
    }
}

pub struct ImageDb<T: Metatada, D: Storage<T>> {
    database: D,
    hasher: ColorMomentHash,
    images: Vec<(Mat, Image<T>)>,
}

impl<T: Metatada, D: Storage<T>> ImageDb<T, D> {
    pub fn new(database: D) -> Self {
        let hasher = ColorMomentHash::new();
        let images = database
            .load_images()
            .into_iter()
            .map(|image| {
                let mat = Mat::image_decode(&image.bytes, ImageReadMode::Color);
                let mat = hasher.compute(&mat);
                (mat, image)
            })
            .collect::<Vec<_>>();
        Self {
            database,
            hasher: hasher,
            images: images,
        }
    }

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

    pub fn image_count(&self) -> usize {
        self.images.len()
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

impl<T: Metatada + serde::Serialize + serde::de::DeserializeOwned> Storage<T> for FileStorage<T> {
    fn save_image(&mut self, image: &Image<T>) {
        let path = self.path.join(image.metadata.file_id());
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

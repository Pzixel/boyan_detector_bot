
use serde;
use serde_json;

use cv::hash::*;
use cv::imgcodecs::*;
use cv::*;
use std::cmp::PartialEq;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::PathBuf;

pub trait Metadata: Clone {
    fn file_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct Image<T: Metadata> {
    pub bytes: Vec<u8>,
    pub metadata: T,
}

impl<T: Metadata> Image<T> {
    pub fn new(bytes: Vec<u8>, metadata: T) -> Self {
        Self { bytes, metadata }
    }
}

pub trait Storage<T: Metadata> {
    fn save_image(&mut self, image: &Image<T>);
    fn load_images(&self) -> Vec<Image<T>>;
}

pub struct InMemoryStorage<T: Metadata> {
    images: Vec<Image<T>>,
}

impl<T: Metadata> InMemoryStorage<T> {
    pub fn new() -> Self {
        Self { images: Vec::new() }
    }
}

impl<T: Metadata> Storage<T> for InMemoryStorage<T> {
    fn save_image(&mut self, image: &Image<T>) {
        self.images.push(image.clone());
    }

    fn load_images(&self) -> Vec<Image<T>> {
        self.images.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ImageVariant<T: Metadata> {
    New,
    AlreadyExists(T),
}

impl<T: Metadata + PartialEq> PartialEq for ImageVariant<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImageVariant::New, ImageVariant::New) => true,
            (ImageVariant::AlreadyExists(a), ImageVariant::AlreadyExists(b)) if a == b => true,
            _ => false,
        }
    }
}

pub struct ImageDb<T: Metadata, D: Storage<T>> {
    database: D,
    hasher: ColorMomentHash,
    images: Vec<(Mat, T)>,
}

impl<T: Metadata, D: Storage<T>> ImageDb<T, D> {
    pub fn new(database: D) -> Self {
        let hasher = ColorMomentHash::new();
        let images = database
            .load_images()
            .into_iter()
            .map(|image| {
                let mat = Mat::image_decode(&image.bytes, ImageReadMode::Color);
                let mat = hasher.compute(&mat);
                (mat, image.metadata)
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
        for &(ref image, ref metadata) in self.images.iter() {
            let diff = self.hasher.compare(&mat, &image);
            if diff < last_diff {
                last_diff = diff;
                result = Some(metadata.clone());
            }
        }
        if last_diff < DIFF {
            return ImageVariant::AlreadyExists(result.unwrap());
        }
        self.database.save_image(&image);
        self.images.push((mat, image.metadata));
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

impl<T: Metadata + serde::Serialize + serde::de::DeserializeOwned> Storage<T> for FileStorage<T> {
    fn save_image(&mut self, image: &Image<T>) {
        let path = self.path.join(image.metadata.file_name());

        let mut binary_file = File::create(&path).unwrap();
        binary_file.write_all(&image.bytes).unwrap();

        let json_file = File::create(path.with_extension("json")).unwrap();
        serde_json::to_writer(json_file, &image.metadata).unwrap();
    }

    fn load_images(&self) -> Vec<Image<T>> {
        let entries = fs::read_dir(&self.path).unwrap();
        entries
            .map(|e| e.unwrap().path())
            .filter(|e| e.extension().and_then(|x| x.to_str()) != Some("json"))
            .map(|path| {
                let mut binary_reader = File::open(&path).unwrap();
                let mut bytes = Vec::with_capacity(1000000);
                binary_reader.read_to_end(&mut bytes).unwrap();

                let reader = File::open(path.with_extension("json")).unwrap();
                let metadata: T = serde_json::from_reader(reader).unwrap();
                Image::new(bytes, metadata)
            })
            .collect()
    }
}

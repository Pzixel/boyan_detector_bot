extern crate cv;

use cv::features2d::*;
use cv::*;
use cv::imgcodecs::*;
use cv::objdetect::*;

pub struct InMemoryDatabase {

}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn insert(&mut self, image: &[u8]) {
        let mat = cv::Mat::image_decode(image, ImageReadMode::Grayscale);
        let mask = Mat::new();
        let mser: SIFT = SIFTBuilder::default().into();
        let (keypoints, descriptors) = mser.detect_and_compute(&mat, &mask);
    }
}
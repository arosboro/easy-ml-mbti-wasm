mod utils;

use wasm_bindgen::prelude::*;

extern crate web_sys;
extern crate easy_ml;
extern crate js_sys;

use easy_ml::matrices::Matrix;
use easy_ml::differentiation::{Record, WengertList};
use easy_ml::linear_algebra;
use easy_ml::numeric::{Numeric};

use std::convert::TryFrom;
use std::convert::TryInto;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, mnist!");
}

const WIDTH: usize = 28;
const HEIGHT: usize = 28;
const TRAINING_SIZE: usize = 8000;
const TESTING_SIZE: usize = 2000;
/// mnist data is grayscale 0-255

type Pixel = u8;

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    data: Vec<Pixel>
}

#[wasm_bindgen]
impl Image {
    /// Creates a new Image
    pub fn new() -> Image {
        Image {
            data: Vec::with_capacity(WIDTH * HEIGHT),
        }
    }

    /// Accesses the data buffer of this Image, for JavaScript to fill with the actual data
    pub fn buffer(&mut self) -> *const Pixel {
        self.data.as_ptr()
    }
}

impl From<Image> for Matrix<f64> {
    fn from(image: Image) -> Self {
        Matrix::from_flat_row_major((28,28), image.data).map(|pixel| (pixel as f64) / 255.0)
    }
}

/// A label type for the MNIST data set, consisting of the 10 digits.
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Digit {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
}

impl TryFrom<u8> for Digit {
    type Error = &'static str;

    fn try_from(integer: u8) -> Result<Self, Self::Error> {
        match integer {
            0 => Ok(Digit::Zero),
            1 => Ok(Digit::One),
            2 => Ok(Digit::Two),
            3 => Ok(Digit::Three),
            4 => Ok(Digit::Four),
            5 => Ok(Digit::Five),
            6 => Ok(Digit::Six),
            7 => Ok(Digit::Seven),
            8 => Ok(Digit::Eight),
            9 => Ok(Digit::Nine),
            _ => Err("Number out of range"),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Dataset {
    images: Vec<Image>,
    labels: Vec<Digit>,
}

#[wasm_bindgen]
impl Dataset {
    pub fn new() -> Dataset {
        Dataset {
            images: Vec::with_capacity(TRAINING_SIZE),
            labels: Vec::with_capacity(TRAINING_SIZE),
        }
    }

    pub fn add(&mut self, image: Image, label: u8) {
        self.images.push(image);
        self.labels.push(label.try_into().expect("Label invalid"));
    }
}

/// A neural network configuration to classify the Mnist data
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct NeuralNetwork {
    weights: Vec<Matrix<f64>>,
    //buffer: Vec<f64>,
}

const FIRST_HIDDEN_LAYER_SIZE: usize = 128;
const SECOND_HIDDEN_LAYER_SIZE: usize = 64;

fn relu<T: Numeric + Copy>(x: T) -> T {
    if x > T::zero() {
        x
    } else {
        T::zero()
    }
}

#[wasm_bindgen]
impl NeuralNetwork {
    /// Creates a new Neural Network configuration of randomised weights
    /// and a simple feed forward architecture.
    pub fn new() -> NeuralNetwork {
        let mut weights = vec![
            Matrix::empty(0.0, (WIDTH * HEIGHT, FIRST_HIDDEN_LAYER_SIZE)),
            Matrix::empty(0.0, (FIRST_HIDDEN_LAYER_SIZE, SECOND_HIDDEN_LAYER_SIZE)),
            Matrix::empty(0.0, (SECOND_HIDDEN_LAYER_SIZE, 10)),
        ];
        for i in 0..weights.len() {
            for j in 0..weights[i].size().0 {
                for k in 0..weights[i].size().1 {
                    weights[i].set(j, k, js_sys::Math::random());
                }
            }
        }
        NeuralNetwork {
            weights,
            //buffer: Vec::with_capacity(0),
        }
    }

    pub fn layers(&self) -> usize {
        self.weights.len()
    }

    pub fn classify(&self, image: &Image) -> Digit {
        let input: Matrix<f64> = image.clone().into();
        // this neural network is a simple feed forward architecture, so dot product
        // the input through the network weights and apply the relu activation
        // function each step, then take softmax to produce an output
        let output = ((input * &self.weights[0]).map(relu) * &self.weights[1]).map(relu) * &self.weights[2];
        let classification = linear_algebra::softmax(output.row_major_iter());
        // find the index of the largest softmax'd label
        classification.iter()
            // find argmax of the output
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("NaN should not be in list"))
            // convert from usize into a Digit
            .map(|(i, _)| i as u8)
            .unwrap()
            .try_into()
            .unwrap()
    }

    // TODO: Would be more informative to plot the neurons?
    // /// Updates and accesses a buffer of a copy of one of the weights, for JavaScript to visualise
    // pub fn get_buffer(&mut self, index: usize) -> *const f64 {
    //     assert!(index < self.weights.len());
    //     self.buffer = self.weights[index].row_major_iter().collect();
    //     self.buffer.as_ptr()
    // }

    pub fn train(&mut self, training_data: &Dataset) {
        let history = WengertList::new();
        let mut training = NeuralNetworkTraining::from(&self, &history);
        training.train(training_data);
        training.update(self);
    }
}

/// At the time of writing, #[wasm_bindgen] does not support lifetimes or type
/// parameters. The Record trait has a lifetime parameter because it must not
/// outlive its WengertList. Unfortunately at the time of writing the WengertList
/// constructor also cannot be a constant function because type parameters other than
/// Sized are not stabalised. Additionally, the WengertList does not implement Sync
/// so it cannot be a shared static variable. The cummulative effect of these restrictions
/// mean that I cannot find a way to pass any structs to JavaScript which include a Record
/// type, even though thread safety is a non concern and any such struct that would be
/// passed to JavaScript would also have been defined to own the WengertList that the Records
/// referenced - ie, such a struct would be completely safe, but I can't find a way to
/// get the Rust type system to agree.
///
/// If you're reading this and #[wasm_bindgen] has added lifetime support, or it's
/// possible to make a WengertList with a &static lifetime, or there's a way to create
/// a struct which owns the WengertList and Records but does not bubble the useless lifetime
/// up then please open an issue or pull request to let me know.
///
/// Until then we will have to not share such types with JavaScript. This is actually
/// not a huge issue, because Records are only needed for training anyway.
#[derive(Clone, Debug)]
struct NeuralNetworkTraining<'a> {
    weights: Vec<Matrix<Record<'a, f64>>>,
}

impl <'a> NeuralNetworkTraining<'a> {
    /// Given a WengertList which will be used exclusively for training this struct,
    /// and an existing configuration for weights, creates a new NeuralNetworkTraining
    fn from(configuration: &NeuralNetwork, history: &'a WengertList<f64>) -> NeuralNetworkTraining<'a> {
        let mut weights = Vec::with_capacity(configuration.weights.len());
        for i in 0..weights.len() {
            weights[i] = Matrix::empty(
                Record::variable(0.0, &history),
                configuration.weights[i].size()
            );
            for j in 0..configuration.weights[i].size().0 {
                for k in 0..configuration.weights[i].size().1 {
                    let neuron = configuration.weights[i].get(j, k);
                    weights[i].get(j, k).number = neuron;
                }
            }
        }
        NeuralNetworkTraining {
            weights,
        }
    }

    /// Updates an existing neural network configuration to the new weights
    /// learned through training.
    fn update(&self, configuration: &mut NeuralNetwork) {
        for i in 0..self.weights.len() {
            for j in 0..self.weights[i].size().0 {
                for k in 0..self.weights[i].size().1 {
                    let neuron = self.weights[i].get(j, k).number;
                    configuration.weights[i].set(j, k, neuron);
                }
            }
        }
    }

    /// Classification is very similar for training, except we stay in floating point
    /// land so we can backprop the error.
    // TODO
    // pub fn classify(&self, image: &Image) -> f64 {
    //     let input: Matrix<f64> = image.clone().into();
    //     // this neural network is a simple feed forward architecture, so dot product
    //     // the input through the network weights and apply the relu activation
    //     // function each step, then take softmax to produce an output
    //     let output = ((input * &self.weights[0]).map(relu) * &self.weights[1]).map(relu) * &self.weights[2];
    //     let classification = linear_algebra::softmax(output.row_major_iter());
    //     // find the index of the largest softmax'd label
    //     classification.iter()
    //         // find argmax of the output
    //         .enumerate()
    //         .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("NaN should not be in list"))
    //         // convert from usize into a Digit
    //         .map(|(i, _)| i as u8)
    //         .unwrap()
    //         .try_into()
    //         .unwrap()
    // }

    fn train(&mut self, training_data: &Dataset) {
        // TODO
    }
}

#[wasm_bindgen]
pub fn prepare() {
    utils::set_panic_hook();
}

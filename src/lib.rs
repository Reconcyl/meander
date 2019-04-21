//! This crate provides a way to show slow change in several variables at once.
//! This change is done in such a way that should explore the value space fairly
//! well while also appearing natural and random.
//! 
//! One place this might be useful is in code that demonstrates how changing certain
//! parameters changes a model.
//! 
//! The variables yielded by this crate will all have values between 0 and 1, so you
//! should scale them to suit your purposes.
//!
//! # How it Works
//!
//! For each variable, there is a separate function that determines its motion.
//! This function is given by the average of three sinusoidal functions.
//! 
//! ```no_run
//! use meander::rand;
//! use meander::typenum::U3;
//! 
//! use meander::Meander;
//! 
//! struct Color {
//!     r: u8,
//!     g: u8,
//!     b: u8,
//! }
//!
//! fn random_colors() -> impl Iterator<Item=Color> {
//!     rand::random::<Meander<U3>>()
//!         .into_time_steps(0.01).map(|a| {
//!             match a.as_slice() {
//!                 // The variables yielded by `Meander` are floats between 0 and 1,
//!                 // so we multiply by 256 and cast to `u8` to get the range we want.
//!                 &[r, g, b] => Color {
//!                     r: (r*256.0) as u8,
//!                     g: (g*256.0) as u8,
//!                     b: (b*256.0) as u8,
//!                 },
//!                 _ => unreachable!()
//!             }
//!         })
//! }
//! ```

#![deny(missing_docs)]

pub use rand;
pub use generic_array;
pub use generic_array::typenum;

use generic_array::{GenericArray, ArrayLength};
use generic_array::functional::FunctionalSequence;
use generic_array::sequence::GenericSequence;

use rand::Rng;
use rand::distributions::{Distribution, Standard};

const PI2: f64 = 2.0 * std::f64::consts::PI;

/// Represents a sinusoid that varies between 0 and 1.
///
/// This can be generated randomly using `rand::random()`.
#[derive(Clone, Copy, Debug)]
pub struct UnitSinusoid {
    /// The number of cycles the function makes per unit time.
    pub frequency: f64,
    /// The location in the cycle the function is `t = 0`.
    pub phase: f64,
}

impl Distribution<UnitSinusoid> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> UnitSinusoid {
        let frequency: f64 = rng.gen_range(1.0, 10.0);
        let phase = rng.gen_range(0.0, frequency.recip());
        UnitSinusoid { frequency, phase }
    }
}

impl UnitSinusoid {
    fn haversin(theta: f64) -> f64 {
        (1.0 - theta.cos()) / 2.0
    }
    /// Find the value of the sinusoid at a given point in time.
    pub fn evaluate(self, t: f64) -> f64 {
        Self::haversin(PI2 * self.frequency * (t + self.phase))
    }
}

/// Represents a curve that meanders through 1-dimensional space. Consists of 3
/// sinusoids whose values are averaged.
///
/// This can be generated randomly using `rand::random()`.
#[derive(Clone, Copy, Debug)]
pub struct Meander1D(pub UnitSinusoid, pub UnitSinusoid, pub UnitSinusoid);

impl Meander1D {
    /// Find the value of the curve at a given point in time.
    pub fn evaluate(self, t: f64) -> f64 {
        ( (self.0).evaluate(t)
        + (self.1).evaluate(t)
        + (self.2).evaluate(t)
        ) / 3.0
    }
}

impl Distribution<Meander1D> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Meander1D {
        Meander1D(rng.gen(), rng.gen(), rng.gen())
    }
}

/// Represents a curve that meanders through `D`-dimensional space.
///
/// This can be generated randomly using `rand::random()`.
#[derive(Clone, Debug)]
pub struct Meander<D: ArrayLength<Meander1D>> {
    /// Each variable is controlled by a separate 1-dimensional function defined here.
    pub curves: GenericArray<Meander1D, D>,
}

impl<D: ArrayLength<Meander1D>> Distribution<Meander<D>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Meander<D> {
        Meander {
            curves: <GenericArray<_, _> as GenericSequence<_>>::generate(|_| rng.gen()),
        }
    }
}

impl<D: ArrayLength<Meander1D> + ArrayLength<f64>> Meander<D> {
    /// Find the value of each of the variables at a particular point in time.
    pub fn evaluate(&self, t: f64) -> GenericArray<f64, D> {
        (&self).curves.clone().map(|c| c.evaluate(t))
    }
    /// Return an iterator yielding the values of the variables at intervals of `dt`.
    pub fn time_steps<'a>(&'a self, dt: f64) -> impl Iterator<Item=GenericArray<f64, D>> + 'a {
        (0..).map(move |i| self.evaluate(i as f64 * dt))
    }
    /// Return an iterator yielding the values of the variables at intervals of `dt`.
    /// Consumes `self`.
    pub fn into_time_steps(self, dt: f64) -> impl Iterator<Item=GenericArray<f64, D>> {
        (0..).map(move |i| self.evaluate(i as f64 * dt))
    }
}

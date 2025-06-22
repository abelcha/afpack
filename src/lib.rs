//! AFPack - CLI tool for managing large dependency folders using ASIF
//! 
//! This crate provides utilities for working with Apple Sparse Image Format (ASIF)
//! and includes a diskimage utility for managing disk images on macOS.

pub mod diskimage;

pub use diskimage::*;
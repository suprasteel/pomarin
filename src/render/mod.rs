//! A module to render the UI for the application
//! This module is split by
//! - config: assets configuration management
//! - egui: ui menus
//! - scene: render 3d objects
//! - shaders
//! - error: errors related to this module
//! - state: wgpu state struct with data available to egui and the objects renderer

pub mod config;
pub mod egui;
pub mod scene;

pub mod error;
pub mod names;
pub mod state;

mod test;

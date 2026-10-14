//! Reusable filesystem browser shared by the app and its portal backend.

pub mod app;
mod browser;
pub mod cli;
mod components;
pub mod context;
mod filesystem;
mod icons;
mod navigation;
pub mod portal;
mod views;
pub mod window;

pub use app::show_window;
pub use window::{WindowMode, WindowRequest};

#[cfg(test)]
mod installation_tests;

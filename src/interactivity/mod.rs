//! Terminal interactivity abstractions including multi-progress tracking and screen rendering.

/// Progress bar utilities backed by `indicatif`.
pub mod progress;
/// Terminal interface controller backed by `dialoguer::console`.
pub mod terminal;

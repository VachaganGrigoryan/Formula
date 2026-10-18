//! The Shorts renderer: a storyboard in, a silent video and its stills out.
//! `tools/shorts` records the narration first and writes the storyboard with
//! each scene's length, then runs `formula-render` on it.

pub mod catalog;
pub mod storyboard;

pub use storyboard::{Cue, Scene, Storyboard, Theme};
pub mod frame;
pub mod layout;

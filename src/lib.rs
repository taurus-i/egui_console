//! A console window for egui applications
//!
//! This crate provides a feature-rich terminal window for egui applications.
//! It's not a shell to the OS; it's simply a command shell window with modern terminal features.

#![warn(missing_docs)]

pub mod console;
mod tab;
mod syntax_highlighting;
mod completion;
mod search;
pub mod koto_integration;
pub mod embeddable_console;

pub use crate::console::ConsoleBuilder;
pub use crate::console::ConsoleEvent;
pub use crate::console::ConsoleWindow;
pub use crate::console::TextStyle;
pub use crate::console::TerminalTheme;
pub use crate::console::StyledText;
pub use crate::syntax_highlighting::Language;
pub use crate::completion::IntelligentCompletion;
pub use crate::koto_integration::{KotoRuntime, KotoError, EguiCommand};
pub use crate::embeddable_console::EmbeddableConsole;

use serde::Serialize;
use strum::EnumIter;

// pub mod api; // todo: uncomment later and adjust api to the code changes
pub mod board;
pub mod coordinates;
pub mod game;
mod movement;
pub mod piece;

/// Terminal and HTML rendering. Not available on wasm: it draws with crossterm.
#[cfg(not(target_family = "wasm"))]
pub mod display;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Serialize, Hash)]
pub enum Side {
    #[default]
    White,
    Black,
}

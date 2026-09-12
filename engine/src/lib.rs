use serde::Serialize;
use std::ops::Not;
use strum::{EnumCount, EnumIter};

pub mod api;
pub mod board;
pub mod coordinates;
/// Terminal and HTML rendering. Not available on wasm: it draws with crossterm.
#[cfg(not(target_family = "wasm"))]
pub mod display;
pub mod game;
mod movement;
pub mod piece;
pub mod zobrist;

#[cfg(target_family = "wasm")]
pub mod wasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter, Serialize, Hash, EnumCount)]
#[repr(u8)]
pub enum Side {
    #[default]
    White = 0,
    Black = 1,
}

impl Side {
    fn move_direction(&self) -> (isize, isize) {
        match self {
            Self::Black => (0, -1),
            Self::White => (0, 1),
        }
    }
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

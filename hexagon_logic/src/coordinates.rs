use serde::Serialize;
use std::fmt::{self};
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CoordinateError {
    #[error("This position is outside the board: x={x}, y={y}")]
    OutsideBoard { y: usize, x: usize },

    #[error("This position is invalid: x={x}, y={y}")]
    InvalidHumanNotation { y: char, x: usize },
}

type Result<T> = std::result::Result<T, CoordinateError>;

pub type HumanNotation = (char, usize); // (a..k | A..K , 1..11)

// Internal range for 2nd Dimension, zero-indexed, open interval
pub const BOARD_DIM: usize = 11;
pub const X_RANGE: [(usize, usize); BOARD_DIM] = [
    (5, 10),
    (4, 10),
    (3, 10),
    (2, 10),
    (1, 10),
    (0, 10),
    (0, 9),
    (0, 8),
    (0, 7),
    (0, 6),
    (0, 5),
];

/// Position is a valid position on the board
///     * zero-indexed
///     * both coordinates are numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Ord, PartialOrd)]
pub struct Position {
    y: usize, // letters in human notation
    x: usize,
}

impl Position {
    /// Creates a Position form a human notation
    pub fn from_human((y, x): HumanNotation) -> Result<Self> {
        if !(1..=BOARD_DIM).contains(&x) {
            return Err(CoordinateError::InvalidHumanNotation { y, x });
        }
        let y = char_to_num_notation(y).ok_or(CoordinateError::InvalidHumanNotation { y, x })?;
        Ok(Self { y, x: x - 1 })
    }

    pub fn new(y: usize, x: usize) -> Result<Self> {
        if !is_on_board(y, x) {
            return Err(CoordinateError::OutsideBoard { y, x });
        }
        Ok(Self { y, x })
    }

    pub(crate) const fn new_const(y: usize, x: usize) -> Self {
        if !is_on_board(y, x) {
            panic!("Invalid start position specified in compile-time constant!");
        }
        Self { y, x }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.y, self.x)
    }

    pub fn to_human(&self) -> HumanNotation {
        let c = num_to_char_notation(self.y).expect("Invalid human notation ???");
        (c, self.x + 1)
    }
}

/// Returns true if this position is inside the board boundaries
const fn is_on_board(y: usize, x: usize) -> bool {
    if y > BOARD_DIM - 1 {
        return false;
    }
    let x_range = X_RANGE[y];

    x >= x_range.0 && x <= x_range.1
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let human = self.to_human();
        _ = write!(f, "Raw: {self:?} === {human:?} ");
        Ok(())
    }
}

pub fn num_to_char_notation(num: usize) -> Option<char> {
    if num >= BOARD_DIM {
        return None;
    }
    let byte = b'A' + num as u8;
    Some(char::from(byte))
}

pub fn char_to_num_notation(y: char) -> Option<usize> {
    if !y.is_ascii() {
        return None;
    }
    let y = y.to_ascii_lowercase();

    let mut y = y as u8;
    if !(b'a'..=b'k').contains(&y) {
        return None;
    }

    y -= b'a';
    Some(y as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let c = Position { y: 1, x: 1 };
        let human: HumanNotation = c.to_human();
        assert_eq!(human, ('B', 2));
    }
}

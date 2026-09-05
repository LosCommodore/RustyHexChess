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

// Internal range for 2nd Dimension, zero-indexed, [min, max[
pub const BOARD_DIM: usize = 11;
pub const X_RANGE: [(usize, usize); BOARD_DIM] = [
    (5, 11),
    (4, 11),
    (3, 11),
    (2, 11),
    (1, 11),
    (0, 11),
    (0, 10),
    (0, 9),
    (0, 8),
    (0, 7),
    (0, 6),
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
    pub fn from_human((y_human, x_human): HumanNotation) -> Result<Self> {
        let error = || CoordinateError::InvalidHumanNotation {
            y: y_human,
            x: x_human,
        };

        let y = char_to_num_notation(y_human).ok_or(error())?;

        let Some(x) = x_human.checked_sub(1) else {
            return Err(error());
        };

        Self::new(y, x)
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

    pub fn coordinates(&self) -> (usize, usize) {
        (self.y, self.x)
    }

    // unique number for each position, 0..91 without gaps.
    // `x` is not zero-based within a row, so the row's first x has to come off.
    pub const fn id(&self) -> usize {
        X_ACCUMULATED[self.y] + self.x - X_RANGE[self.y].0
    }
}

/// Returns true if this position is inside the board boundaries
const fn is_on_board(y: usize, x: usize) -> bool {
    if y > BOARD_DIM - 1 {
        return false;
    }
    let x_range = X_RANGE[y];

    x >= x_range.0 && x < x_range.1
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

// helper function to give each hexagon field a unique 1d-number
const fn acc_indexes<const N: usize>() -> [usize; N] {
    let mut x_acc = [0usize; N];
    let mut state = 0;
    let mut i = 0;

    // Standard `while` loops are perfectly valid in const fns
    while i < N {
        // the offset of this row's first field, so its own length is not included yet
        x_acc[i] = state;

        let (x_min, x_max) = X_RANGE[i];
        let x_len = x_max - x_min;
        state += x_len;

        i += 1;
    }

    x_acc
}

const X_ACCUMULATED: [usize; BOARD_DIM] = acc_indexes();

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn to_human() {
        let c = Position { y: 1, x: 1 };
        let human: HumanNotation = c.to_human();
        assert_eq!(human, ('B', 2));
    }

    #[should_panic]
    #[test]
    fn new_invalid_position() {
        let _ = Position::new(100, 100).unwrap();
    }

    // Every field gets its own id, and the ids fill 0..91 without gaps, so they can
    // index a table with one slot per field.
    #[test]
    fn ids_are_unique_and_dense() {
        let all: Vec<Position> = (0..BOARD_DIM)
            .flat_map(|y| (0..BOARD_DIM).filter_map(move |x| Position::new(y, x).ok()))
            .collect();

        assert_eq!(all.len(), 91, "the board has 91 fields");

        let ids: HashSet<usize> = all.iter().map(|pos| pos.id()).collect();

        assert_eq!(ids.len(), all.len(), "two fields share an id");
        assert_eq!(ids.iter().min(), Some(&0));
        assert_eq!(ids.iter().max(), Some(&(all.len() - 1)));
    }
}

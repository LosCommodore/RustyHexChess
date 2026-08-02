use std::fmt::{self};

use anyhow::{Result, bail};

/// Internal range for 2nd Dimension, zero-indexed, open interval

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

pub type HumanCoordinates = (char, usize); // [a..k] and [1..11]

/// Internal positions, 0-indexed | both coordinates are numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub y: usize, // letters
    pub x: usize,
}

impl Position {
    /// Returns true if this position is inside the board boundaries
    pub fn is_on_board(&self) -> bool {
        if self.y > BOARD_DIM - 1 {
            return false;
        }
        let x_range = X_RANGE[self.y];

        self.x >= x_range.0 && self.x <= x_range.1
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ok(human) = HumanCoordinates::try_from(self.clone()) else {
            return write!(f, "ERROR, invalid coordinate");
        };
        _ = write!(f, "Raw: {self:?} === {human:?} ");
        Ok(())
    }
}

impl TryFrom<HumanCoordinates> for Position {
    type Error = anyhow::Error;

    fn try_from((y, x): HumanCoordinates) -> Result<Self, Self::Error> {
        let y = char_to_num_notation(y)?;
        Ok(Self { y, x: x - 1 })
    }
}

impl TryFrom<Position> for HumanCoordinates {
    type Error = anyhow::Error;

    fn try_from(p: Position) -> Result<Self, Self::Error> {
        let c = num_to_char_notation(p.y)?;
        Ok((c, p.x + 1))
    }
}

pub fn num_to_char_notation(num: usize) -> Result<char> {
    let code = num + 65; // 65 == ASCII('A')
    let c = char::from(u8::try_from(code)?);
    Ok(c)
}

pub fn char_to_num_notation(y: char) -> Result<usize> {
    if !y.is_ascii() {
        bail!("First item of position must be ascii")
    }
    let y = y.to_ascii_lowercase();

    let mut y = y as usize;
    if !(97..=107).contains(&y) {
        bail!("First item of position must be between 'a' and 'k'")
    }

    y -= 97; // ASCII('a') -> 0
    Ok(y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let c = Position { y: 1, x: 1 };
        let human: HumanCoordinates = c.try_into().unwrap();
        assert_eq!(human, ('B', 2));
    }
}

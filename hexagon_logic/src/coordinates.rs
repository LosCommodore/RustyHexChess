use std::fmt::{self};

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

pub type HumanCoordinate = (char, usize); // (a..k | A..K , 1..11)

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
        let Ok(human) = HumanCoordinate::try_from(self.clone()) else {
            return write!(f, "ERROR, invalid coordinate");
        };
        _ = write!(f, "Raw: {self:?} === {human:?} ");
        Ok(())
    }
}

impl TryFrom<HumanCoordinate> for Position {
    type Error = ();

    fn try_from((y, x): HumanCoordinate) -> Result<Self, Self::Error> {
        if !(1..=BOARD_DIM).contains(&x) {
            return Err(());
        }
        let y = char_to_num_notation(y).ok_or(())?;
        Ok(Self { y, x: x - 1 })
    }
}

impl TryFrom<Position> for HumanCoordinate {
    type Error = ();

    fn try_from(p: Position) -> Result<Self, Self::Error> {
        let c = num_to_char_notation(p.y).ok_or(())?;
        Ok((c, p.x + 1))
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
        let human: HumanCoordinate = c.try_into().unwrap();
        assert_eq!(human, ('B', 2));
    }
}

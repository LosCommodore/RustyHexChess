use anyhow::{Ok, Result, bail};

pub type HumanCoordinates = (char, usize); // e.g. ('a',1) -> see doc folder

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub y: usize, // letters
    pub x: usize,
}

impl Position {
    #[allow(unused)]
    pub fn to_human(&self) -> Result<HumanCoordinates> {
        let c = num_to_char_notation(self.y)?;
        Ok((c, self.x + 1))
    }

    pub fn from_human((y, x): HumanCoordinates) -> Result<Self> {
        let y = char_to_num_notation(y)?;
        Ok(Self { y, x: x - 1 })
    }
}

pub fn num_to_char_notation(num: usize) -> Result<char> {
    let mut code = num + 65;

    if code >= 74 {
        // skip J (74)
        code += 1;
    }
    let c = char::from(u8::try_from(code)?);
    Ok(c)
}

pub fn char_to_num_notation(y: char) -> Result<usize> {
    // TODO: Check if coordinates are on board.

    if !y.is_ascii() {
        bail!("First item of position must be ascii")
    }
    let y = y.to_ascii_lowercase();

    if y == 'j' {
        bail!("Letter 'j' is not used!")
    }

    let mut y = y as usize; // a = 97
    if !(97..=108).contains(&y) {
        bail!("First item of position must be between 'a' and 'l'")
    }

    if y > 106 {
        y -= 1; // skip J
    }

    y -= 97;
    Ok(y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let c = Position { y: 1, x: 1 };
        let human = c.to_human().unwrap();

        assert_eq!(human, ('B', 2));

        let c = Position { y: 10, x: 5 };
        let human = c.to_human().unwrap();
        assert_eq!(human, ('L', 6));
    }

    #[test]
    fn to_internal_skips_j() {
        let internal = Position::from_human(('K', 1)).unwrap();
        assert_eq!(internal, Position { y: 9, x: 0 });

        let internal = Position::from_human(('L', 1)).unwrap();
        assert_eq!(internal, Position { y: 9, x: 0 });
    }
}

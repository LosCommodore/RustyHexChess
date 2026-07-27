use anyhow::{Ok, Result, bail};

pub type HumanCoordinates = (char, usize); // e.g. ('a',1) -> [a..k] and [1..11]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub y: usize, // letters
    pub x: usize,
}

impl TryFrom<HumanCoordinates> for Position {
    type Error = anyhow::Error;
    
    fn try_from((y, x): HumanCoordinates) -> Result<Self, Self::Error> {
             let y = char_to_num_notation(y)?;
        Ok(Self { y, x })
    }
}

impl TryFrom<Position> for HumanCoordinates {
    type Error = anyhow::Error;
    
    fn try_from(p: Position) -> Result<Self, Self::Error> {
    let c = num_to_char_notation(p.y)?;
        Ok((c, p.x))
    }

}

pub fn num_to_char_notation(num: usize) -> Result<char> {
    let mut code = num + 64;
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

    y -= 96; // ascii(a) -> 1
    Ok(y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let c = Position { y: 1, x: 1 };
        let human: HumanCoordinates =  c.try_into().unwrap();

        assert_eq!(human, ('B', 2));

        let c = Position { y: 10, x: 5 };
        let human: HumanCoordinates = c.try_into().unwrap();
        assert_eq!(human, ('L', 6));
    }
}

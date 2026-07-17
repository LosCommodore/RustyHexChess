use anyhow::{Ok, Result, bail};

pub type HumanCoordinates = (char, usize); // e.g. ('a',1) -> see doc folder
pub type InternalCoordinates = (usize, usize); // 0-indexed

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

#[allow(unused)]
pub fn to_human_coordinates((y, x): InternalCoordinates) -> Result<HumanCoordinates> {
    let c = num_to_char_notation(y)?;
    Ok((c, x + 1))
}

pub fn to_internal_coordinates((y, x): HumanCoordinates) -> Result<InternalCoordinates> {
    let y = char_to_num_notation(y)?;
    Ok((y, x - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_human() {
        let human = to_human_coordinates((1, 1)).unwrap();
        assert_eq!(human, ('B', 2));

        let human = to_human_coordinates((10, 5)).unwrap();
        assert_eq!(human, ('L', 6));
    }

    #[test]
    fn to_internal_skips_j() {
        let internal = to_internal_coordinates(('K', 1)).unwrap();
        assert_eq!(internal, (9, 0));

        let internal = to_internal_coordinates(('L', 1)).unwrap();
        assert_eq!(internal, (10, 0));
    }
}

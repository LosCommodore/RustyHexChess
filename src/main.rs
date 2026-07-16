mod display;
mod game;
use anyhow::Result;
use game::Piece;

use crate::display::print_board;

fn main() -> Result<()> {
    let _piece = Piece::new(('A', 1), game::PieceType::King, game::PlayerColor::Black)?;

    print_board()?;
    Ok(())
}

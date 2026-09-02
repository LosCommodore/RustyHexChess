//! JavaScript bindings for [`crate::api`].
//!
//! Deliberately thin: every method parses its arguments, calls one `GameApi`
//! method, and serializes the result. All rules, validation and wire formats
//! live in `api.rs`, which is plain Rust and tested natively — this file only
//! carries values across the boundary, so there is nothing here that can only
//! be exercised in a browser.
//!
//! Values cross as JSON that is parsed into real JS objects by `JSON.parse`
//! before it is handed over, which keeps the interface free of
//! `serde-wasm-bindgen` while still giving callers plain objects that match the
//! declared TypeScript types. Failures are thrown as JS `Error`s carrying an
//! extra `code` property, so callers can branch on `err.code` instead of
//! matching on message text.

use wasm_bindgen::prelude::*;

use crate::api::{ApiError, ApiErrorDto, Color, GameApi, Kind, PlacedPiece};

/// The only TypeScript still written by hand: everything else in `engine.d.ts`
/// is generated from the `api.rs` types by `tsify`, so it cannot drift.
///
/// `Square` is an alias this crate has no Rust type for, and `HexChessError`
/// describes a thrown JS `Error` rather than a value that crosses the boundary
/// — but its `code` is `ErrorCode`, which *is* generated.
#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/** A square in the engine's notation, file a-k and rank 1-11, e.g. "f5". */
export type Square = string;

/** Thrown by every failing call; `code` is stable, `message` is for humans. */
export interface HexChessError extends Error {
  code: ErrorCode;
}
"#;

/// A game of hexagonal chess. Create one, then drive it with `play`, `promote`
/// and `undo`; each returns the full state that results.
#[wasm_bindgen]
pub struct HexChess {
    api: GameApi,
}

#[wasm_bindgen]
impl HexChess {
    /// A new game from the standard starting position.
    #[wasm_bindgen(constructor)]
    pub fn new() -> HexChess {
        HexChess {
            api: GameApi::new(),
        }
    }

    /// A new game from a position set up by hand.
    ///
    /// Both kings must be on the board, or the position is refused.
    #[wasm_bindgen(js_name = fromPieces)]
    pub fn from_pieces(
        #[wasm_bindgen(unchecked_param_type = "PlacedPiece[]")] pieces: JsValue,
        #[wasm_bindgen(unchecked_param_type = "Color")] active: &str,
    ) -> Result<HexChess, JsValue> {
        let pieces: Vec<PlacedPiece> = from_js(pieces)?;
        let active: Color = active.parse().map_err(throw)?;

        Ok(HexChess {
            api: GameApi::from_pieces(&pieces, active).map_err(throw)?,
        })
    }

    /// The current position.
    #[wasm_bindgen(unchecked_return_type = "GameState")]
    pub fn state(&mut self) -> Result<JsValue, JsValue> {
        to_js(self.api.state().map_err(throw)?)
    }

    /// Where the piece on `square` may go. Empty for a square the player to
    /// move cannot move from, so it is safe to call on any click.
    #[wasm_bindgen(js_name = legalMoves, unchecked_return_type = "LegalMove[]")]
    pub fn legal_moves(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Square")] square: &str,
    ) -> Result<JsValue, JsValue> {
        to_js(self.api.legal_moves(square).map_err(throw)?)
    }

    /// Plays a move. Throws on an illegal one, leaving the game untouched.
    #[wasm_bindgen(unchecked_return_type = "GameState")]
    pub fn play(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Square")] from: &str,
        #[wasm_bindgen(unchecked_param_type = "Square")] to: &str,
    ) -> Result<JsValue, JsValue> {
        to_js(self.api.play(from, to).map_err(throw)?)
    }

    /// Replaces the pawn waiting on the far rank. Only valid while the phase
    /// is `"promotion"`.
    #[wasm_bindgen(unchecked_return_type = "GameState")]
    pub fn promote(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Kind")] kind: &str,
    ) -> Result<JsValue, JsValue> {
        let kind: Kind = kind.parse().map_err(throw)?;
        to_js(self.api.promote(kind).map_err(throw)?)
    }

    /// Takes back the last move, including one that ended the game.
    #[wasm_bindgen(unchecked_return_type = "GameState")]
    pub fn undo(&mut self) -> Result<JsValue, JsValue> {
        to_js(self.api.undo().map_err(throw)?)
    }

    /// Returns to the standard starting position.
    #[wasm_bindgen(unchecked_return_type = "GameState")]
    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.api.reset();
        to_js(self.api.state().map_err(throw)?)
    }
}

impl Default for HexChess {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializes a wire type and parses it back into a JS object, so callers get
/// the shape the TypeScript declares rather than a string to parse themselves.
fn to_js<T: serde::Serialize>(value: T) -> Result<JsValue, JsValue> {
    let text = serde_json::to_string(&value).map_err(|e| {
        throw(ApiError::Engine {
            message: format!("could not serialize the result: {e}"),
        })
    })?;

    js_sys::JSON::parse(&text)
}

/// The reverse: a JS value in, a wire type out.
fn from_js<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    let text = js_sys::JSON::stringify(&value)
        .map(String::from)
        .map_err(|_| {
            throw(ApiError::Engine {
                message: "the argument could not be read as JSON".to_string(),
            })
        })?;

    serde_json::from_str(&text).map_err(|e| {
        throw(ApiError::Engine {
            message: format!("the argument has the wrong shape: {e}"),
        })
    })
}

/// Turns an [`ApiError`] into a JS `Error` that also carries its `code`.
fn throw(error: ApiError) -> JsValue {
    let dto = ApiErrorDto::from(&error);
    let code: &'static str = dto.code.into();
    let js_error = js_sys::Error::new(&dto.message);
    js_error.set_name("HexChessError");
    let _ = js_sys::Reflect::set(&js_error, &"code".into(), &code.into());
    js_error.into()
}

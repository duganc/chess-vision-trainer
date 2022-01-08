use pleco::tools::eval::Eval;
use pleco::board::{Board as PlecoBoard};

use crate::board::{Board, Side};


pub struct Evaluator();

impl Evaluator {

    pub fn evaluate(board: &Board, side: Side) -> i32 {
        Eval::eval_low(&Self::from(board, side, 1, 1))
    }

    fn from(board: &Board, side: Side, half_moves: usize, full_moves: usize) -> PlecoBoard {
        PlecoBoard::from_fen(&board.fen(side, half_moves, full_moves)).unwrap()
    }
}
use rand::{seq::IteratorRandom, thread_rng};
use rand::rngs::ThreadRng;
use rand::prelude::*;
use crate::board::{Board, Move, Square, Side, Rank, File};


#[derive(Debug)]
pub struct Game {
	board: Board,
	next_to_act: Side,
	moves: Vec<Move>,
	rng: ThreadRng
}

impl Game {

	pub fn new() -> Self {
		let board = Board::starting_position();
		let next_to_act = Side::White;
		let moves = Vec::new();
		Self {
			board,
			next_to_act,
			moves,
			rng: thread_rng()
		}
	}

	pub fn get_next_to_act(&self) -> Side {
		self.next_to_act
	}

	pub fn pretty_print_moves(&self) -> String {
		let mut to_return = "".to_string();
		let mut side = Side::White;
		for m in &self.moves {
			let move_string = format!("{: >5}", self.board.get_move_string(*m));
			let delimiter = match side {
				Side::White => "|".to_string(),
				Side::Black => "\n".to_string()
			};
			to_return += &(move_string + &delimiter);
			side = Side::get_opponent(side);
		}
		return to_return.to_string();
	}

	pub fn is_game_over(&self) -> bool {
		Side::all().into_iter().any(|s| self.is_stalemate_by(s) || self.is_won_by(s))
	}

	pub fn is_stalemate_by(&self, side: Side) -> bool {
		self.board.is_stalemated(Side::get_opponent(side))
	}

	pub fn is_won_by(&self, side: Side) -> bool {
		self.board.is_checkmated(Side::get_opponent(side))
	}

	pub fn make_move(&mut self, m: Move) {
		self.board.make_move(m);
		self.moves.push(m);
		self.next_to_act = Side::get_opponent(self.next_to_act);
	}

	pub fn make_moves(&mut self, moves: Vec<Move>) {
		for m in moves {
			self.make_move(m);
		}
	}

	pub fn make_moves_from_string(&mut self, s: String) {
		let s = str::replace(&s, "\n", "");
		let s = str::replace(&s, "\t", "");
		let s = str::replace(&s, " ", "");
		let s = str::replace(&s, "x", "");
		let s = str::replace(&s, "+", "");
		let s = str::replace(&s, "#", "");

		let move_strings = s.split(",");
		let mut side = self.next_to_act;
		for move_string in move_strings {
			let m = self.board.force_parse_move(side, move_string);
			self.board.make_move(m);
			side = Side::get_opponent(side);
		}
	}

	pub fn make_random_move(&mut self) {
		let moves = self.board.get_legal_moves_for_side(self.next_to_act);
		let result = moves.iter().choose(&mut self.rng);
		match result {
			Some(m) => self.make_move(*m),
			None => panic!("No legal moves remaining!")
		};
		println!("{:?} made move: {:?}", Side::get_opponent(self.next_to_act), result);
	}

	pub fn make_random_moves(&mut self, n: usize) {
		for _i in 0..n {
			self.make_random_move();
		}
	}

	pub fn make_random_moves_and_end_on_random_side(&mut self, rounds: usize) {
		let mut n = 2*rounds;
		let adjust: bool = self.rng.gen();
		if adjust {
			n = n - 1;
		}
		self.make_random_moves(n);
	}

}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn game_smoke_test() {
		let mut game = Game::new();
		assert!(!game.is_won_by(Side::White));
		assert!(!game.is_won_by(Side::Black));

		game.make_moves_from_string(
			"
			d4,
			d5,
			Bf4,
			Nf6,
			Nf3,
			Nh5,
			Be5,
			f6,
			Bg3,
			f5,
			e3,
			Nd7,
			Bd3,
			Ndf6,
			Nc3,
			e6,
			O-O,
			Bd6,
			Bh4,
			f4,
			exf4,
			Bxf4,
			Ne5,
			Qd6,
			Re1,
			h6,
			Bxf6,
			gxf6,
			Qxh5,
			Kd8,
			Nf7,
			Kd7,
			Nxd6,
			cxd6,
			Qf7,
			Kc6,
			Qxf6,
			Bd7,
			Qxf4,
			Raf8,
			Bb5,
			Kc7,
			Qh4,
			Bc8,
			Qe7,
			Kb8,
			Rxe6,
			Bxe6,
			Qxe6,
			Rh7,
			Nxd5,
			a6,
			Bd7,
			Rd8,
			Qxd6,
			Ka8,
			Nb6,
			Ka7,
			Nc8,
			Ka8,
			Qb6,
			Rhxd7,
			Qa7
			".to_string()
		);
		assert!(game.is_won_by(Side::White));
	}
}
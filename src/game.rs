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

	pub fn get_board_clone(&self) -> Board {
		self.board.clone()
	}

	pub fn pretty_print_moves(&self) -> String {
		let mut to_return = "".to_string();
		let mut side = Side::White;
		let mut replay_board = Board::starting_position();
		for m in &self.moves {
			let move_string = format!("{: >5}", replay_board.get_move_string(*m));
			let delimiter = match side {
				Side::White => "|".to_string(),
				Side::Black => "\n".to_string()
			};
			to_return += &(move_string + &delimiter);
			replay_board.make_move(*m);
			side = Side::get_opponent(side);
		}
		return to_return.to_string();
	}

	pub fn pretty_print_board(&self) -> String {
		self.board.pretty_print()
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
		let move_strings = Move::parse_move_strings(s);
		let mut side = self.next_to_act;
		for move_string in move_strings {
			let m = self.board.force_parse_move(side, &move_string);
			self.make_move(m);
			side = Side::get_opponent(side);
		}
	}

	pub fn get_checks(&self) -> Vec<Move> {
		self.board.get_checks(self.next_to_act)
	}

	pub fn get_captures(&self) -> Vec<Move> {
		self.board.get_captures(self.next_to_act)
	}

	pub fn parse_moves_from_current_position(&self, s: String) -> Result<Vec<Move>, String> {
		let move_results: Vec<Result<Move, String>> = Move::parse_move_strings(s).into_iter().map(|m| self.board.try_parse_move(self.next_to_act, &m)).collect();
		let n_errors = move_results.iter().filter(|m| m.is_err()).count();
		if n_errors > 0 {
			let first_error: Option<Result<Move, String>> = move_results.into_iter().filter(|m| m.is_err()).nth(0);
			return match first_error {
				None => panic!("Unreachable as we've shown that there's at least one error above."),
				Some(Err(result)) => Err(result),
				Some(Ok(_)) => panic!("Unreachable as we've shown that there's at least one error above."),
			};
		} else {
			return Ok(move_results.into_iter().map(|m| m.unwrap()).collect());
		}
	}

	pub fn parse_sequential_moves(&self, s: String) -> Result<Vec<Move>, String> {
		let move_strings = Move::parse_move_strings(s);
		let mut parsing_board = self.get_board_clone();
		let mut side = self.next_to_act;
		let mut to_return = Vec::new();
		for move_string in move_strings {
			match parsing_board.try_parse_move(side, &move_string) {
				Ok(m) => {
					parsing_board.make_move(m);
					to_return.push(m);
				},
				Err(e) => {return Err(e)}
			};
			side = Side::get_opponent(side);
		}

		return Ok(to_return);

	}

	pub fn get_move_string_from_current_position(&self, m: Move) -> String {
		self.board.get_move_string(m)
	}

	pub fn get_move_strings_from_current_position(&self, moves: Vec<Move>) -> String {
		let move_strings: Vec<String> = moves.iter().map(|x| self.get_move_string_from_current_position(*x)).collect();
		let mut to_return = "".to_string();
		if move_strings.len() == 0 {
			return to_return;
		}
		for move_string in move_strings {
			to_return += &format!("{}{}", move_string, ", ".to_string());
		}
		return to_return[0..(to_return.len()) - 2].to_string();
	}

	pub fn make_random_move(&mut self) {
		let moves = self.board.get_legal_moves_for_side(self.next_to_act);
		let result = moves.iter().choose(&mut self.rng);
		match result {
			Some(m) => self.make_move(*m),
			None => panic!("No legal moves remaining!")
		};
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

	#[test]
	fn game_pretty_prints_moves() {
		let moves = "
		Nh3,
		b5,
		Ng1,
		Nf6,
		g3,
		Ne4,
		b3,
		Nd2,
		h3,
		g5,
		h4,
		c5,
		Bd2,
		c4,
		c3,
		d5,
		e3,
		Ba6,
		Bh3,
		h5
		".to_string();
		let mut game = Game::new();
		game.make_moves_from_string(moves);
		assert!(game.board != Board::starting_position());

		assert_eq!(
			game.pretty_print_moves(),
			 "  Nh3|   b5\n".to_string() + 
			&"  Ng1|  Nf6\n".to_string() +
			&"   g3|  Ne4\n".to_string() +
			&"   b3|  Nd2\n".to_string() +
			&"   h3|   g5\n".to_string() +
			&"   h4|   c5\n".to_string() +
			&"  Bd2|   c4\n".to_string() +
			&"   c3|   d5\n".to_string() +
			&"   e3|  Ba6\n".to_string() +
			&"  Bh3|   h5\n".to_string());
	}
}
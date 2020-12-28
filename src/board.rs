use std::ops::BitAnd;
use std::ops::BitOr;
use std::convert::TryInto;
use regex::Regex;

lazy_static! {
	static ref FORWARD_PAWN_MOVE: Regex = Regex::new("^([a-h])([1-8])$").unwrap();
	static ref DISAMBIGUATED_PAWN_MOVE: Regex = Regex::new("^([a-h])([a-h])([1-8])$").unwrap();
	static ref PIECE_MOVE: Regex = Regex::new("^([N,B,R,Q,K])([a-h])([1-8])$").unwrap();
	static ref DISAMBIGUATED_PIECE_MOVE: Regex = Regex::new("^([a-h])([N,B,R,Q,K])([a-h])([1-8])$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Board {
	white: Bitboard,
	black: Bitboard,
	pawns: Bitboard,
	knights: Bitboard,
	bishops: Bitboard,
	rooks: Bitboard,
	queens: Bitboard,
	kings: Bitboard,
	prior_move_white: Option<Move>,
	prior_move_black: Option<Move>,
	castling_rights_white_kingside: bool,
	castling_rights_white_queenside: bool,
	castling_rights_black_kingside: bool,
	castling_rights_black_queenside: bool,
}

impl Board {

	pub fn empty() -> Self {
		let white = Bitboard::empty();
		let black = Bitboard::empty();
		let pawns = Bitboard::empty();
		let knights = Bitboard::empty();
		let bishops = Bitboard::empty();
		let rooks = Bitboard::empty();
		let queens = Bitboard::empty();
		let kings = Bitboard::empty();
		Self {
			white,
			black,
			pawns,
			knights,
			bishops,
			rooks,
			queens,
			kings,
			prior_move_white: None,
			prior_move_black: None,
			castling_rights_white_kingside: true,
			castling_rights_white_queenside: true,
			castling_rights_black_kingside: true,
			castling_rights_black_queenside: true,
		}
	}

	pub fn starting_position() -> Self {
		let white = Bitboard::rank(Rank::One) | Bitboard::rank(Rank::Two);
		let black = Bitboard::rank(Rank::Seven) | Bitboard::rank(Rank::Eight);

		let pawns = Bitboard::rank(Rank::Two) | Bitboard::rank(Rank::Seven);

		let kings = Bitboard::from_squares(vec![Square::new(File::E, Rank::One), Square::new(File::E, Rank::Eight)]);
		let queens = Bitboard::from_squares(vec![Square::new(File::D, Rank::One), Square::new(File::D, Rank::Eight)]);
		let rooks = Bitboard::from_squares(
			vec![
				Square::new(File::A, Rank::One),
				Square::new(File::H, Rank::One),
				Square::new(File::A, Rank::Eight),
				Square::new(File::H, Rank::Eight),
			]
			
		);
		let bishops = Bitboard::from_squares(
			vec![
				Square::new(File::C, Rank::One),
				Square::new(File::F, Rank::One),
				Square::new(File::C, Rank::Eight),
				Square::new(File::F, Rank::Eight),
			]
		);
		let knights = Bitboard::from_squares(
			vec![
				Square::new(File::B, Rank::One),
				Square::new(File::G, Rank::One),
				Square::new(File::B, Rank::Eight),
				Square::new(File::G, Rank::Eight),
			]
		);

		Self {
			white,
			black,
			pawns,
			kings,
			queens,
			rooks,
			bishops,
			knights,
			prior_move_white: None,
			prior_move_black: None,
			castling_rights_white_kingside: true,
			castling_rights_white_queenside: true,
			castling_rights_black_kingside: true,
			castling_rights_black_queenside: true,
		}
	}

	pub fn add(&mut self, side: Side, piece: Piece, square: Square) {
		if self.is_occupied(square) {
			panic!("{:?} is already occupied!", square);
		}

		let bb = Bitboard::square(square);

		match side {
			Side::White => {self.white = self.white | bb;},
			Side::Black => {self.black = self.black | bb;},
		};

		match piece {
			Piece::Pawn => {self.pawns = self.pawns | bb;},
			Piece::Knight => {self.knights = self.knights | bb;},
			Piece::Bishop => {self.bishops = self.bishops | bb;},
			Piece::Rook => {self.rooks = self.rooks | bb;},
			Piece::Queen => {self.queens = self.queens | bb;},
			Piece::King => {self.kings = self.kings | bb;},
		};

	}

	pub fn is_occupied(&self, square: Square) -> bool {
		self.pieces().is_occupied(square)
	}


	pub fn get(&self, square: Square) -> Option<(Side, Piece)> {
		match self.is_occupied(square) {
			false => None,
			true => {
				let bb = Bitboard::square(square);
				let side;
				if (self.white & bb).has_pieces() {
					side = Side::White;
				} else if (self.black & bb).has_pieces() {
					side = Side::Black;
				} else {
					panic!("{:?} is occupied but is neither white nor black.", square);
				}

				let piece;
				if (self.pawns & bb).has_pieces() {
					piece = Piece::Pawn;
				} else if (self.knights & bb).has_pieces() {
					piece = Piece::Knight;
				} else if (self.bishops & bb).has_pieces() {
					piece = Piece::Bishop;
				} else if (self.rooks & bb).has_pieces() {
					piece = Piece::Rook;
				} else if (self.queens & bb).has_pieces() {
					piece = Piece::Queen;
				} else if (self.kings & bb).has_pieces() {
					piece = Piece::King;
				} else {
					panic!("{:?} is occupied but is no piece type.", square);
				}

				Some((side, piece))
			}
		}
	}

	pub fn is_in_check(&self, side: Side) -> bool {
		let kings = self.get_side_pieces_bitboard(side, Piece::King).to_squares();
		let opponent = Side::get_opponent(side);
		return kings.iter().any(|k| self.is_attacking(opponent, *k));
	}

	pub fn is_attacking(&self, attacker: Side, square: Square) -> bool {
		
		let immediate_diagonals = self.get_immediately_diagonal_vision(square);
		if (immediate_diagonals & self.get_side_pieces_bitboard(attacker, Piece::Pawn)).has_pieces() {
			return true;
		}

		let adjacent = self.get_adjacent_vision(square);
		if (adjacent & self.get_side_pieces_bitboard(attacker, Piece::King)).has_pieces() {
			return true;
		}

		let knight_moves = self.get_knight_vision(square);
		if (knight_moves & self.get_side_pieces_bitboard(attacker, Piece::Knight)).has_pieces() {
			return true;
		}

		let diagonals = self.get_diagonal_vision(square);
		if (diagonals & self.get_side_pieces_bitboard(attacker, Piece::Bishop)).has_pieces() || (diagonals & self.get_side_pieces_bitboard(attacker, Piece::Queen)).has_pieces() {
			return true;
		}

		let laterals = self.get_lateral_vision(square);
		if (laterals & self.get_side_pieces_bitboard(attacker, Piece::Rook)).has_pieces() || (laterals & self.get_side_pieces_bitboard(attacker, Piece::Queen)).has_pieces() {
			return true;
		}

		return false;
	}

	pub fn make_move(&mut self, m: Move) {
		assert!(self.is_legal_move(m), "{:?} isn't a legal move.", m);
		self.force_make_move(m);
	}

	fn force_make_move(&mut self, m: Move) {
		let source = m.0;
		let (side, _piece) = self.get(source).unwrap();
		let kingside_castle = Castle::Kingside.get_king_move(side);
		let queenside_castle = Castle::Queenside.get_king_move(side);
		if m == kingside_castle {
			let king_move = m;
			let rook_move = Castle::Kingside.get_rook_move(side);
			self.transform(king_move);
			self.transform(rook_move);
		} else if m == queenside_castle {
			let king_move = m;
			let rook_move = Castle::Queenside.get_rook_move(side);
			self.transform(king_move);
			self.transform(rook_move);
		} else {
			self.transform(m);
		}
	}

	pub fn get_transformation(&self, m: Move) -> Self {
		let mut to_return = self.clone();
		to_return.transform(m);
		return to_return;
	}

	pub fn transform(&mut self, m: Move) {
		let source = m.0;
		let (side, piece) = match self.get(source) {
			Some(p) => p,
			_ => panic!("There's no piece on {:?}!", source)
		};

		let side_bb = self.get_side_bitboard(side);
		let side_bb = side_bb.transform(m);

		let pieces_bb = self.get_pieces_bitboard(piece);
		let pieces_bb = pieces_bb.transform(m);

		self.set_side_bitboard(side, side_bb);
		self.set_pieces_bitboard(piece, pieces_bb);
		self.disambiguate_captures(side, piece);
	}

	fn disambiguate_captures(&mut self, side: Side, piece: Piece) {
		let side_bb = self.get_side_bitboard(side);
		let opponent = Side::get_opponent(side);
		let opponent_bb = self.get_side_bitboard(opponent);

		self.set_side_bitboard(opponent, opponent_bb & (side_bb.get_inverse()));

		for p in Piece::all() {
			self.priviledge_piece(piece, p);
		}
	}

	fn priviledge_piece(&mut self, priviledged: Piece, to_overwrite: Piece) {
		if priviledged == to_overwrite {
			return;
		}

		let priviledged_bb = self.get_pieces_bitboard(priviledged);
		let to_overwrite_bb = self.get_pieces_bitboard(to_overwrite);

		self.set_pieces_bitboard(to_overwrite, to_overwrite_bb & (priviledged_bb.get_inverse()));
	}

	pub fn force_parse_move(&self, side: Side, r#move: &str) -> Move {
		self.try_parse_move(side, r#move).unwrap()
	}

	pub fn try_parse_move(&self, side: Side, r#move: &str) -> Result<Move, String> {
		
		match r#move {
			"O-O-O" => {
				let castles = self.get_castles(side);
				if castles.contains(&Castle::Queenside) {
					return Ok(Castle::Queenside.get_king_move(side));
				} else {
					return Err("Queenside castling is invalid in this position!".to_string());
				}
			},
			"O-O" => {
				let castles = self.get_castles(side);
				if castles.contains(&Castle::Kingside) {
					return Ok(Castle::Kingside.get_king_move(side));
				} else {
					return Err("Queenside castling is invalid in this position!".to_string());
				}
			},
			m => {
				if FORWARD_PAWN_MOVE.is_match(m) {
					let characters = FORWARD_PAWN_MOVE.captures(m).unwrap();
					let destination = Square::new(
						File::from_str(characters.get(1).map_or("", |m| m.as_str())),
						Rank::from_str(characters.get(2).map_or("", |m| m.as_str()))
					);
					
					if self.is_occupied(destination) {
						return Err(format!("Pawn cannot move forward onto an occupied space!  Move: {:?}", m));
					}
					let source_result = self.get_pawn_move_from_square_in_front(side, destination);
					return source_result.map(|source| Move::new(source, destination));
				} else if DISAMBIGUATED_PAWN_MOVE.is_match(m) {
					let characters = DISAMBIGUATED_PAWN_MOVE.captures(m).unwrap();
					let destination = Square::new(
						File::from_str(characters.get(2).map_or("", |m| m.as_str())),
						Rank::from_str(characters.get(3).map_or("", |m| m.as_str()))
					);
					let source_file = File::from_str(characters.get(1).map_or("", |m| m.as_str()));

					let square_in_front = Square::new(source_file, destination.1);
					let source_result = self.get_pawn_move_from_square_in_front(side, square_in_front);
					match source_result {
						Err(s) => return Err(s),
						Ok(source) => {
							match Rank::distance(source.1, destination.1) {
								2 => {
									// TODO check en passant rights
									return Ok(Move::new(source, destination));
								},
								1 => return Ok(Move::new(source, destination)),
								_ => panic!("get_source_pawn_from_square_in_front({:?}, {:?}) returned an invalid move: {:?}", side, destination, source)
							};
						}
					}
					
				} else if PIECE_MOVE.is_match(m) {
					let characters = PIECE_MOVE.captures(m).unwrap();
					let destination = Square::new(
						File::from_str(characters.get(2).map_or("", |m| m.as_str())),
						Rank::from_str(characters.get(3).map_or("", |m| m.as_str()))
					);
					let piece = Piece::from_str(characters.get(1).map_or("", |m| m.as_str()));

					if (self.get_side_bitboard(side) & Bitboard::square(destination)).has_pieces() {
						return Err(format!("{:?} cannot move to an occupied square {:?}.", piece, destination));
					}

					let source_result = self.get_unambiguous_piece_source(side, piece, destination);

					match source_result {
						Err(e) => return Err(e),
						Ok(source) => {
							return Ok(Move::new(source, destination));
						}
					}
					
				} else {
					return Err(format!("Invalid or unsupported move: {:?}", m));
				}
			}
		}
	}

	fn get_unambiguous_piece_source(&self, side: Side, piece: Piece, destination: Square) -> Result<Square, String> {
		let pieces = self.get_side_pieces_bitboard(side, piece);
		let potential_sources = match piece {
			Piece::Pawn => panic!("get_unambiguous_piece_source can't be used on Pawns.  Use get_pawn_move_from_square_in_front instead."),
			Piece::Knight => (self.get_knight_vision(destination) & pieces).to_squares(),
			Piece::Bishop => (self.get_diagonal_vision(destination) & pieces).to_squares(),
			Piece::Rook => (self.get_lateral_vision(destination) & pieces).to_squares(),
			Piece::Queen => ((self.get_diagonal_vision(destination) | self.get_lateral_vision(destination)) & pieces).to_squares(),
			Piece::King => (self.get_adjacent_vision(destination) & pieces).to_squares(),
		};
		if potential_sources.len() == 0 {
			return Err(format!("No {:?} {:?} can reach the destination {:?}.  Board:\n{:?}", side, piece, destination, self.pieces().print()));
		} else if potential_sources.len() > 2 {
			return Err(format!("Ambiguous potential sources for {:?} {:?}: {:?}.  Board:\n{:?}", side, piece, potential_sources, self.pieces().print()));
		} else {
			let source = potential_sources[0];
			return Ok(source);
		}
	}

	fn get_pawn_move_from_square_in_front(&self, side: Side, destination: Square) -> Result<Square, String> {
		let backwards = Direction::get_backward(side);
		let one_back = destination.get_adjacent(backwards);
		match one_back {
			None => {return Err(format!("Pawns can't be on the first rank!  Destination: {:?}", destination))},
			Some(s) => {
				let pawns = self.get_side_pieces_bitboard(side, Piece::Pawn);
				if pawns.is_occupied(s) {
					let source = s;
					return Ok(source);
				} else {
					let two_back = s.get_adjacent(backwards);
					match two_back {
						None => {return Err(format!("Pawns can't move to the second rank!  Destination: {:?}", destination));},
						Some(t) => {
							if pawns.is_occupied(t) {
								return Ok(t);
							} else {
								return Err(format!("There are no pawns behind the destination square! Destination: {:?}", destination));
							}
							
						}
					}
				}
			}
		}
	}

	fn get_side_pieces_bitboard(&self, side: Side, piece: Piece) -> Bitboard {
		self.get_side_bitboard(side) & self.get_pieces_bitboard(piece)
	}

	fn get_side_bitboard(&self, side: Side) -> Bitboard {
		match side {
			Side::White => self.white,
			Side::Black => self.black
		}
	}

	fn set_side_bitboard(&mut self, side: Side, bitboard: Bitboard) {
		match side {
			Side::White => {self.white = bitboard},
			Side::Black => {self.black = bitboard}
		};
	}

	fn get_pieces_bitboard(&self, piece: Piece) -> Bitboard {
		match piece {
			Piece::Pawn => self.pawns,
			Piece::Knight => self.knights,
			Piece::Bishop => self.bishops,
			Piece::Rook => self.rooks,
			Piece::Queen => self.queens,
			Piece::King => self.kings,
		}
	}

	fn set_pieces_bitboard(&mut self, piece: Piece, bitboard: Bitboard) {
		match piece {
			Piece::Pawn => {self.pawns = bitboard},
			Piece::Knight => {self.knights = bitboard},
			Piece::Bishop => {self.bishops = bitboard},
			Piece::Rook => {self.rooks = bitboard},
			Piece::Queen => {self.queens = bitboard},
			Piece::King => {self.kings = bitboard},
		}
	}

	pub fn is_legal_move(&self, m: Move) -> bool {
		self.get_legal_moves(m.0).contains(&m)
	}


	pub fn get_legal_moves(&self, square: Square) -> Vec<Move> {
		let (side, piece) = match self.get(square) {
			Some(p) => p,
			_ => panic!("There's no piece on {:?}!", square)
		};

		let to_return = match piece {
			Piece::Pawn => {
				let forward_pawn_moves = self.get_forward_pawn_moves(side, square);
				self.get_en_passant_takes(side, square) | self.get_diagonal_pawn_takes(side, square) | forward_pawn_moves
			},
			Piece::Knight => {
				self.get_knight_moves(side, square)
			}
			Piece::Bishop => {
				self.get_diagonal_moves(side, square)
			}
			Piece::Rook => {
				self.get_lateral_moves(side, square)
			}
			Piece::Queen => {
				self.get_diagonal_moves(side, square) | self.get_lateral_moves(side, square)
			}
			Piece::King => {
				let castles = self.get_castles(side);
				self.get_adjacent_moves(side, square) | Bitboard::from_squares(castles.into_iter().map(|c| c.get_king_destination(side)).collect())
			}
		};

		let to_return = self.get_moves_not_into_check(square, to_return);
		return to_return;
	}

	fn get_moves_not_into_check(&self, source: Square, bitboard: Bitboard) -> Vec<Move> {
		let destinations = bitboard.to_squares();
		let (side, piece) = match self.get(source) {
			Some(p) => p,
			_ => panic!("There's no piece on {:?}!", source)
		};
		return destinations.into_iter()
			.filter(|destination| !self.get_transformation(Move::new(source, *destination)).is_in_check(side))
			.map(|destination| Move::new(source, destination))
			.collect();

	}

	fn get_en_passant_takes(&self, side: Side, square: Square) -> Bitboard {
		let prior_move = match side {
			Side::White => self.prior_move_white,
			Side::Black => self.prior_move_black
		};
		Bitboard::empty() // TODO
	}

	fn get_diagonal_pawn_takes(&self, side: Side, square: Square) -> Bitboard {
		let directions = Direction::get_forward_diagonals(side);

		let potential_captures = match (square.get_adjacent(directions.0), square.get_adjacent(directions.1)) {
			(None, None) => Bitboard::empty(),
			(None, Some(r)) => Bitboard::square(r),
			(Some(l), None) => Bitboard::square(l),
			(Some(l), Some(r)) => Bitboard::from_squares(vec![l, r])
		};
		let opponent_pieces = self.get_side_bitboard(Side::get_opponent(side));
		return potential_captures & opponent_pieces;
	}

	fn get_forward_pawn_moves(&self, side: Side, square: Square) -> Bitboard {
		let direction = Direction::get_forward(side);
		let mut to_return = match square.get_adjacent(direction) {
			None => Bitboard::empty(),
			Some(s) => {
				match self.is_occupied(s) {
					true => Bitboard::empty(),
					false => Bitboard::square(s)
				}
			}
		};
		let is_first_pawn_move = square.1 == Rank::first_pawn_move(side);
		if is_first_pawn_move && (to_return.has_pieces()) {
			match square.get_adjacent(direction).map(|x| x.get_adjacent(direction)).flatten() {
				None => { panic!("Pawn moving two steps would run off the board!  {:?}", square); }
				Some(s) => {
					match self.is_occupied(s) {
						true => {},
						false => {
							to_return = to_return | Bitboard::square(s);
						}
					}
				}
			};
		}

		return to_return;

	}

	fn get_castles(&self, side: Side) -> Vec<Castle> {
		let rank = Castle::get_rank(side);
		let is_king_on_e = (self.get_side_pieces_bitboard(side, Piece::King) & Bitboard::file(File::E)).has_pieces();
		if !is_king_on_e {
			return Vec::new();
		}

		let rooks = self.get_side_pieces_bitboard(side, Piece::Rook);
		let is_kingside_rook_in_place = (rooks & Bitboard::square(Square::new(File::H, rank))).has_pieces();
		let is_queenside_rook_in_place = (rooks & Bitboard::square(Square::new(File::A, rank))).has_pieces();

		let kingside_blockers = vec![
			Square::new(File::F, rank),
			Square::new(File::G, rank),
		];
		let is_kingside_blocked = (Bitboard::from_squares(kingside_blockers) & self.pieces()).has_pieces();

		let queenside_blockers = vec![
			Square::new(File::B, rank),
			Square::new(File::C, rank),
			Square::new(File::D, rank),
		];
		let is_queenside_blocked = (Bitboard::from_squares(queenside_blockers) & self.pieces()).has_pieces();

		let opponent = Side::get_opponent(side);
		let kingside_king_squares = vec![
			Square::new(File::E, rank),
			Square::new(File::F, rank),
			Square::new(File::G, rank),
		];
		let is_kingside_attacked = kingside_king_squares.iter().any(|x| self.is_attacking(opponent, *x));

		let queenside_king_squares = vec![
			Square::new(File::C, rank),
			Square::new(File::D, rank),
			Square::new(File::E, rank),
		];
		let is_queenside_attacked = queenside_king_squares.iter().any(|x| self.is_attacking(opponent, *x));

		let (castling_rights_kingside, castling_rights_queenside) = match side {
			Side::White => (self.castling_rights_white_kingside, self.castling_rights_white_queenside),
			Side::Black => (self.castling_rights_black_kingside, self.castling_rights_black_queenside),
		};

		let can_castle_kingside = is_kingside_rook_in_place && is_kingside_blocked && is_kingside_attacked && castling_rights_kingside;
		let can_castle_queenside = is_queenside_rook_in_place && is_queenside_blocked && is_queenside_attacked && castling_rights_queenside;

		match (can_castle_kingside, can_castle_queenside) {
			(false, false) => Vec::new(),
			(true, false) => vec![Castle::Kingside],
			(false, true) => vec![Castle::Queenside],
			(true, true) => vec![Castle::Kingside, Castle::Queenside]
		}

	}

	fn get_adjacent_moves(&self, side: Side, square: Square) -> Bitboard {
		self.get_adjacent_vision(square) & self.get_side(side).get_inverse()
	}

	fn get_knight_moves(&self, side: Side, square: Square) -> Bitboard {
		self.get_knight_vision(square) & self.get_side(side).get_inverse()
	}

	fn get_diagonal_moves(&self, side: Side, square: Square) -> Bitboard {
		self.get_diagonal_vision(square) & self.get_side(side).get_inverse()
	}

	fn get_lateral_moves(&self, side: Side, square: Square) -> Bitboard {
		self.get_lateral_vision(square) & self.get_side(side).get_inverse()
	}

	fn get_diagonal_vision(&self, square: Square) -> Bitboard {
		let up_left = self.get_vision_directional(square, Direction::UpLeft);
		let up_right = self.get_vision_directional(square, Direction::UpRight);
		let down_left = self.get_vision_directional(square, Direction::DownLeft);
		let down_right = self.get_vision_directional(square, Direction::DownRight);
		up_left | up_right | down_left | down_right
	}

	fn get_immediately_diagonal_vision(&self, square: Square) -> Bitboard {
		let squares = Direction::diagonals().iter().map(|d| square.get_adjacent(*d)).filter(|x| !x.is_none()).map(|x| x.unwrap()).collect();
		Bitboard::from_squares(squares)
	}

	fn get_adjacent_vision(&self, square: Square) -> Bitboard {
		let squares = Direction::all().iter().map(|d| square.get_adjacent(*d)).filter(|x| !x.is_none()).map(|x| x.unwrap()).collect();
		Bitboard::from_squares(squares)
	}

	fn get_knight_vision(&self, square: Square) -> Bitboard {
		let squares = Direction::knight_moves().iter().map(|moves| square.get_relative(moves.to_vec())).filter(|x| !x.is_none()).map(|x| x.unwrap()).collect();
		Bitboard::from_squares(squares)
	}

	fn get_lateral_vision(&self, square: Square) -> Bitboard {
		self.get_file_vision(square) | self.get_rank_vision(square)
	}

	fn get_file_vision(&self, square: Square) -> Bitboard {
		let right = self.get_vision_directional(square, Direction::Right);
		let left = self.get_vision_directional(square, Direction::Left);
		right | left
	}

	fn get_rank_vision(&self, square: Square) -> Bitboard {
		let up = self.get_vision_directional(square, Direction::Up);
		let down = self.get_vision_directional(square, Direction::Down);
		up | down
	}

	fn get_pieces_on_file(&self, file: File) -> Bitboard {
		self.pieces() & Bitboard::file(file)
	}

	fn get_pieces_on_rank(&self, rank: Rank) -> Bitboard {
		self.pieces() & Bitboard::rank(rank)
	}

	fn get_vision_directional(&self, square: Square, direction: Direction) -> Bitboard {
		let mut file = Some(square.0);
		let mut rank = Some(square.1);
		let mut to_return = Bitboard::empty();
		while (!file.is_none()) && (!rank.is_none()) {
			let (f, r) = match direction {
				Direction::Up => (file, rank.unwrap().next()),
				Direction::Down => (file, rank.unwrap().previous()),
				Direction::Left => (file.unwrap().previous(), rank),
				Direction::Right => (file.unwrap().next(), rank),
				Direction::UpLeft => (file.unwrap().previous(), rank.unwrap().next()),
				Direction::UpRight => (file.unwrap().next(), rank.unwrap().next()),
				Direction::DownLeft => (file.unwrap().previous(), rank.unwrap().previous()),
				Direction::DownRight => (file.unwrap().previous(), rank.unwrap().next()),
			};
			file = f;
			rank = r;
			if f.is_none() || r.is_none() {
				break;
			}
			let test_square = Square(file.unwrap(), rank.unwrap());
			to_return = to_return | Bitboard::square(test_square);
			if self.is_occupied(test_square) {
				break;
			}
		}

		return to_return;
	}


	fn get_side(&self, side: Side) -> Bitboard {
		match side {
			Side::White => self.white,
			Side::Black => self.black
		}
	}

	fn get_opponent(&self, side: Side) -> Bitboard {
		match side {
			Side::White => self.black,
			Side::Black => self.white
		}
	}

	fn pieces(&self) -> Bitboard {
		self.white | self.black
	}

}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Castle {
	Kingside,
	Queenside
}

impl Castle {

	pub fn get_king_move(&self, side: Side) -> Move {
		let source = Square::new(File::E, Self::get_rank(side));
		let destination = self.get_king_destination(side);
		Move::new(source, destination)
	}

	pub fn get_rook_move(&self, side: Side) -> Move {
		let source = self.get_rook_source(side);
		let destination = self.get_rook_destination(side);
		Move::new(source, destination)
	}

	pub fn get_rank(side: Side) -> Rank {
		match side {
			Side::White => Rank::One,
			Side::Black => Rank::Eight
		}
	}

	fn get_king_destination(&self, side: Side) -> Square {
		let rank = Self::get_rank(side);
		let file = match self {
			Self::Kingside => File::G,
			Self::Queenside => File::C
		};
		Square(file, rank)
	}

	fn get_rook_source(&self, side: Side) -> Square {
		let rank = Self::get_rank(side);
		let file = match self {
			Self::Kingside => File::H,
			Self::Queenside => File::A
		};
		Square(file, rank)
	}

	fn get_rook_destination(&self, side: Side) -> Square {
		let rank = Self::get_rank(side);
		let file = match self {
			Self::Kingside => File::F,
			Self::Queenside => File::D
		};
		Square(file, rank)
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Side {
	White,
	Black
}

impl Side {

	pub fn get_opponent(side: Side) -> Self {
		match side {
			Side::White => Side::Black,
			Side::Black => Side::White
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Piece {
	Pawn,
	Knight,
	Bishop,
	Rook,
	Queen,
	King
}

impl Piece {

	pub fn from_str(s: &str) -> Self {
		assert_eq!(s.len(), 1);
		let c = s.chars().nth(0).unwrap();
		Self::from_char(c)
	}

	pub fn from_char(c: char) -> Self {
		match c {
			'P' => Piece::Pawn,
			'N' => Piece::Knight,
			'B' => Piece::Bishop,
			'R' => Piece::Rook,
			'Q' => Piece::Queen,
			'K' => Piece::King,
			_ => panic!("Invalid character: {:?}", c)
		}
	}

	pub fn all() -> Vec<Self> {
		vec![
			Piece::Pawn,
			Piece::Knight,
			Piece::Bishop,
			Piece::Rook,
			Piece::Queen,
			Piece::King
		]
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Move(Square, Square);

impl Move {

	pub fn new(from: Square, to: Square) -> Self {
		Move(from, to)
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Square(File, Rank);

impl Square {

	pub fn new(file: File, rank: Rank) -> Self {
		Square(file, rank)
	}

	pub fn from_string(s: &str) -> Self {
		if s.len() != 2 {
			panic!("s should be 2 long but is {:?} ({:?})", s.len(), s);
		}

		let file_char = s.chars().nth(0).unwrap();
		let rank_char = s.chars().nth(1).unwrap();

		let file = File::from_char(file_char);
		let rank = Rank::from_char(rank_char);

		Square(file, rank)
	}

	pub fn get_adjacent(&self, direction: Direction) -> Option<Self> {
		let file = Some(self.0);
		let rank = Some(self.1);
		let (file, rank) = match direction {
			Direction::Up => (file, rank.unwrap().next()),
			Direction::Down => (file, rank.unwrap().previous()),
			Direction::Left => (file.unwrap().previous(), rank),
			Direction::Right => (file.unwrap().next(), rank),
			Direction::UpLeft => (file.unwrap().previous(), rank.unwrap().next()),
			Direction::UpRight => (file.unwrap().next(), rank.unwrap().next()),
			Direction::DownLeft => (file.unwrap().previous(), rank.unwrap().previous()),
			Direction::DownRight => (file.unwrap().next(), rank.unwrap().previous()),
		};
		
		return match (file, rank) {
			(None, _) => None,
			(_, None) => None,
			(Some(f), Some(r)) => Some(Square(f, r))
		};
	}

	pub fn get_relative(&self, directions: Vec<Direction>) -> Option<Self> {
		directions.iter().fold(Some(*self), |accumulator, d| accumulator.map(|x| x.get_adjacent(*d)).flatten())
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
	UpLeft,
	UpRight,
	DownLeft,
	DownRight
}

impl Direction {

	fn all() -> Vec<Self> {
		vec![
			Direction::Up,
			Direction::Down,
			Direction::Left,
			Direction::Right,
			Direction::UpLeft,
			Direction::UpRight,
			Direction::DownLeft,
			Direction::DownRight,
		]
	}

	fn diagonals() -> Vec<Self> {
		vec![
			Direction::UpLeft,
			Direction::UpRight,
			Direction::DownLeft,
			Direction::DownRight,
		]
	}


	fn knight_moves() -> Vec<Vec<Self>> {
		vec![
			vec![Direction::Up, Direction::UpLeft],
			vec![Direction::Up, Direction::UpRight],
			vec![Direction::Right, Direction::UpRight],
			vec![Direction::Right, Direction::DownRight],
			vec![Direction::Down, Direction::DownRight],
			vec![Direction::Down, Direction::DownLeft],
			vec![Direction::Left, Direction::DownLeft],
			vec![Direction::Left, Direction::UpLeft],
		]
	}

	fn get_forward(side: Side) -> Self {
		match side {
			Side::White => Direction::Up,
			Side::Black => Direction::Down
		}
	}

	fn get_backward(side: Side) -> Self {
		match side {
			Side::White => Direction::Down,
			Side::Black => Direction::Up
		}
	}

	fn get_forward_diagonals(side: Side) -> (Self, Self) {
		match side {
			Side::White => (Direction::UpLeft, Direction::UpRight),
			Side::Black => (Direction::DownLeft, Direction::DownRight)
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum File {
	A = 0,
	B = 1,
	C = 2,
	D = 3,
	E = 4,
	F = 5,
	G = 6,
	H = 7
}

impl File {

	pub fn from_string(s: String) -> Self {
		if s.len() != 1 {
			panic!("Invalid string length: {:?}", s);
		}
		Self::from_char(s.chars().nth(0).unwrap())
	}

	pub fn from_str(s: &str) -> Self {
		if s.len() != 1 {
			panic!("Invalid string length: {:?}", s);
		}
		Self::from_char(s.chars().nth(0).unwrap())
	}

	pub fn from_char(s: char) -> Self {
		s.to_ascii_lowercase();
		match s {
			'a' => File::A,
			'b' => File::B,
			'c' => File::C,
			'd' => File::D,
			'e' => File::E,
			'f' => File::F,
			'g' => File::G,
			'h' => File::H,
			_ => panic!("Invalid File: {:?}", s)
		}
	}

	pub fn all() -> Vec<Self> {
		vec![
			File::A,
			File::B,
			File::C,
			File::D,
			File::E,
			File::F,
			File::G,
			File::H,
		]
	}

	pub fn previous(&self) -> Option<Self> {
		match *self {
			File::A => None,
			f => Some(File::from_u64((f as u64) - 1))
		}
	}

	pub fn next(&self) -> Option<Self> {
		match *self {
			File::H => None,
			f => Some(File::from_u64((f as u64) + 1))
		}
	}

	pub fn from_u64(i: u64) -> Self {
		match i {
			0 => File::A,
			1 => File::B,
			2 => File::C,
			3 => File::D,
			4 => File::E,
			5 => File::F,
			6 => File::G,
			7 => File::H,
			_ => panic!("Invalid File: {:?}", i)
		}
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Rank {
	One = 0,
	Two = 1,
	Three = 2,
	Four = 3,
	Five = 4,
	Six = 5,
	Seven = 6,
	Eight = 7
}

impl Rank {

	pub fn from_string(s: String) -> Self {
		if s.len() != 1 {
			panic!("Invalid string length: {:?}", s);
		}
		Self::from_char(s.chars().nth(0).unwrap())
	}

	pub fn from_str(s: &str) -> Self {
		if s.len() != 1 {
			panic!("Invalid string length: {:?}", s);
		}
		Self::from_char(s.chars().nth(0).unwrap())
	}

	pub fn from_char(s: char) -> Self {
		match s {
			'1' => Rank::One,
			'2' => Rank::Two,
			'3' => Rank::Three,
			'4' => Rank::Four,
			'5' => Rank::Five,
			'6' => Rank::Six,
			'7' => Rank::Seven,
			'8' => Rank::Eight,
			_ => panic!("Invalid Rank: {:?}", s)
		}
	}

	pub fn all() -> Vec<Self> {
		vec![
			Rank::One,
			Rank::Two,
			Rank::Three,
			Rank::Four,
			Rank::Five,
			Rank::Six,
			Rank::Seven,
			Rank::Eight,
		]
	}

	pub fn previous(&self) -> Option<Self> {
		match *self {
			Rank::One => None,
			r => Some(Rank::from_u64((r as u64) - 1))
		}
	}

	pub fn next(&self) -> Option<Self> {
		match *self {
			Rank::Eight => None,
			r => Some(Rank::from_u64((r as u64) + 1))
		}
	}

	pub fn from_u64(i: u64) -> Self {
		match i {
			0 => Rank::One,
			1 => Rank::Two,
			2 => Rank::Three,
			3 => Rank::Four,
			4 => Rank::Five,
			5 => Rank::Six,
			6 => Rank::Seven,
			7 => Rank::Eight,
			_ => panic!("Invalid Rank: {:?}", i)
		}
	}

	pub fn distance(source: Rank, destination: Rank) -> u64 {
		((source as i8) - (destination as i8)).abs().try_into().unwrap()
	}

	pub fn first_pawn_move(side: Side) -> Self {
		match side {
			Side::White => Rank::Two,
			Side::Black => Rank::Seven
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bitboard(u64);

impl Bitboard {

	pub fn empty() -> Self {
		Bitboard(0)
	}

	pub fn from_squares(squares: Vec<Square>) -> Self {
		squares.iter().fold(Self::empty(), |accumulator, x| accumulator | Self::square(*x))
	}

	pub fn square(square: Square) -> Self {
		Self::file(square.0) & Self::rank(square.1)
	}

	pub fn to_squares(&self) -> Vec<Square> {
		let mut to_return = Vec::new();
		for file in File::all() {
			for rank in Rank::all() {
				let square = Square::new(file, rank);
				if (&Self::square(square) & self).has_pieces() {
					to_return.push(square);
				}
			}
		}
		return to_return;
	}

	pub fn rank(rank: Rank) -> Self {
		Bitboard(255 << (8*(rank as u64))) // 255 = 1+2+4+8+16+32+64+128 = 2**8 - 1
	}

	pub fn file(file: File) -> Self {
		Bitboard(72340172838076673 << (file as u64)) // 72340172838076673 = 1+2**8+2**16+2**24+2**32+2**40+2**48+2**56
	}

	pub fn is_empty(&self) -> bool {
		self.0 == 0
	}

	pub fn has_pieces(&self) -> bool {
		!self.is_empty()
	}

	pub fn is_occupied(&self, square: Square) -> bool {
		(&Self::square(square) & self).0 > 0
	}

	pub fn get_inverse(&self) -> Self {
		Bitboard(!self.0)
	}

	pub fn transform(&self, m: Move) -> Self {
		let source = m.0;
		let destination = m.1;

		if !self.is_occupied(m.0) {
			panic!("{:?} isn't occupied!", source);
		}

		return (self & &(Bitboard::square(source).get_inverse())) | Bitboard::square(destination);
	}

	pub fn print(&self) -> String {
		
		let horizontal_border = "+---+---+---+---+---+---+---+---+\n".to_string();
		let mut to_return = horizontal_border.clone();
		for rank in Rank::all().into_iter().rev() {
			let rank_bb = Self::rank(rank);
			let squares: Vec<&str> = File::all().into_iter().map(|file| if (self & &rank_bb & Self::file(file)).has_pieces() { "#" } else { " " }).collect();
			to_return += &format!("| {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} | {:?} |\n", squares[0].to_string(), squares[1].to_string(), squares[2].to_string(), squares[3].to_string(), squares[4].to_string(), squares[5].to_string(), squares[6].to_string(), squares[7]).to_string();
			to_return += &horizontal_border;
		}
		return to_return.to_string();
	}

}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAnd for &Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAnd for &mut Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOr for &Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOr for &mut Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn board_smoke_test() {
		let mut board = Board::empty();

		board.add(Side::White, Piece::Pawn, Square::new(File::E, Rank::Four));
		board.add(Side::Black, Piece::Pawn, Square::from_string("e5"));
		board.add(Side::White, Piece::King, Square::from_string("e1"));
		board.add(Side::Black, Piece::King, Square::from_string("e8"));
		board.add(Side::White, Piece::Knight, Square::from_string("g7"));

		assert!(board.is_legal_move(Move::new(Square::from_string("g7"), Square::from_string("h5"))));
		assert!(board.is_legal_move(board.try_parse_move(Side::White, "Ke2").unwrap()));
		assert!(board.try_parse_move(Side::White, "Ka1").is_err());
		assert!(board.try_parse_move(Side::White, "e5").is_err());
		assert!(board.is_in_check(Side::Black));
		assert!(!board.is_in_check(Side::White));

		assert!(board.is_attacking(Side::White, Square::from_string("d5")));
		assert!(!board.is_attacking(Side::Black, Square::from_string("d5")));
		assert!(board.is_attacking(Side::White, Square::from_string("f5")));
		assert!(!board.is_attacking(Side::Black, Square::from_string("f5")));

		assert!(board.is_attacking(Side::White, Square::from_string("e6")));
		assert!(!board.is_attacking(Side::Black, Square::from_string("e6")));
	}

	#[test]
	fn board_initializes_starting_position() {
		let board = Board::starting_position();

		assert_eq!(board.pieces().to_squares().len(), 32);

		assert_eq!(board.get_side_pieces_bitboard(Side::White, Piece::King).to_squares(), vec![Square::from_string("e1")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::White, Piece::Queen).to_squares(), vec![Square::from_string("d1")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::White, Piece::Rook).to_squares(), vec![Square::from_string("a1"), Square::from_string("h1")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::White, Piece::Bishop).to_squares(), vec![Square::from_string("c1"), Square::from_string("f1")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::White, Piece::Knight).to_squares(), vec![Square::from_string("b1"), Square::from_string("g1")]);
		let white_pawns = board.get_side_pieces_bitboard(Side::White, Piece::Pawn).to_squares();
		assert_eq!(white_pawns.len(), 8);
		assert!(white_pawns.iter().all(|x| x.1 == Rank::Two));


		assert_eq!(board.get_side_pieces_bitboard(Side::Black, Piece::King).to_squares(), vec![Square::from_string("e8")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::Black, Piece::Queen).to_squares(), vec![Square::from_string("d8")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::Black, Piece::Rook).to_squares(), vec![Square::from_string("a8"), Square::from_string("h8")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::Black, Piece::Bishop).to_squares(), vec![Square::from_string("c8"), Square::from_string("f8")]);
		assert_eq!(board.get_side_pieces_bitboard(Side::Black, Piece::Knight).to_squares(), vec![Square::from_string("b8"), Square::from_string("g8")]);
		let black_pawns = board.get_side_pieces_bitboard(Side::Black, Piece::Pawn).to_squares();
		assert_eq!(black_pawns.len(), 8);
		assert!(black_pawns.iter().all(|x| x.1 == Rank::Seven));
	}

	#[test]
	fn board_parses_moves() {
		let mut board = Board::empty();
		board.add(Side::White, Piece::Pawn, Square::from_string("e4"));
		assert_eq!(board.try_parse_move(Side::White, "e5").unwrap(), Move::new(Square::from_string("e4"), Square::from_string("e5")));

		board.add(Side::Black, Piece::Pawn, Square::from_string("f5"));
		assert_eq!(board.try_parse_move(Side::Black, "f4").unwrap(), Move::new(Square::from_string("f5"), Square::from_string("f4")));
		assert_eq!(board.try_parse_move(Side::Black, "fe4").unwrap(), Move::new(Square::from_string("f5"), Square::from_string("e4")));

		board.add(Side::Black, Piece::Pawn, Square::from_string("h7"));
		assert_eq!(board.try_parse_move(Side::Black, "h5").unwrap(), Move::new(Square::from_string("h7"), Square::from_string("h5")));

	}

	#[test]
	fn board_gets_legal_moves() {
		let mut board = Board::empty();
		board.add(Side::Black, Piece::Pawn, Square::from_string("f5"));
		let moves = board.get_legal_moves(Square::from_string("f5"));
		assert_eq!(moves[0], Move(Square::from_string("f5"), Square::from_string("f4")), "{:?}", moves);

		board.add(Side::White, Piece::Queen, Square::from_string("e4"));
		board.add(Side::White, Piece::Knight, Square::from_string("g4"));

		let pawn_takes = board.get_diagonal_pawn_takes(Side::Black, Square::from_string("f5"));
		assert_eq!(pawn_takes.to_squares().len(), 2);

		let moves = board.get_legal_moves(Square::from_string("f5"));
		assert_eq!(moves.len(), 3, "{:?}", moves);
		assert_eq!(moves[0], Move(Square::from_string("f5"), Square::from_string("e4")), "{:?}", moves);
		assert_eq!(moves[1], Move(Square::from_string("f5"), Square::from_string("f4")), "{:?}", moves);
		assert_eq!(moves[2], Move(Square::from_string("f5"), Square::from_string("g4")), "{:?}", moves);

		board.add(Side::White, Piece::Pawn, Square::from_string("a2"));
		let moves = board.get_legal_moves(Square::from_string("a2"));
		assert_eq!(
			moves,
			vec![
				Move(Square::from_string("a2"), Square::from_string("a3")),
				Move(Square::from_string("a2"), Square::from_string("a4")),
			]
		)
	}

	#[test]
	fn board_computes_castles() {
		let mut board = Board::starting_position();

		let white_king_square = Square::from_string("e1");
		let white_king_moves = board.get_legal_moves(white_king_square);
		let black_king_square = Square::from_string("e8");
		let black_king_moves = board.get_legal_moves(black_king_square);

		assert!(&board.try_parse_move(Side::White, "O-O").is_err());
		assert!(&board.try_parse_move(Side::Black, "O-O").is_err());
		assert!(&board.try_parse_move(Side::White, "O-O-O").is_err());
		assert!(&board.try_parse_move(Side::Black, "O-O-O").is_err());

		assert!(!white_king_moves.contains(&Castle::Kingside.get_king_move(Side::White)));
		assert!(!black_king_moves.contains(&Castle::Kingside.get_king_move(Side::Black)));
		assert!(!white_king_moves.contains(&Castle::Queenside.get_king_move(Side::White)));
		assert!(!black_king_moves.contains(&Castle::Queenside.get_king_move(Side::Black)));

		board.make_move(board.force_parse_move(Side::White, "e4"));
		board.make_move(board.force_parse_move(Side::Black, "d6"));
		board.make_move(board.force_parse_move(Side::White, "Nf3"));
		board.make_move(board.force_parse_move(Side::Black, "Nf6"));

		let white_king_moves = board.get_legal_moves(white_king_square);
		let black_king_moves = board.get_legal_moves(black_king_square);

		assert!(&board.try_parse_move(Side::White, "O-O").is_err());
		assert!(&board.try_parse_move(Side::Black, "O-O").is_err());
		assert!(&board.try_parse_move(Side::White, "O-O-O").is_err());
		assert!(&board.try_parse_move(Side::Black, "O-O-O").is_err());

		assert!(!white_king_moves.contains(&Castle::Kingside.get_king_move(Side::White)));
		assert!(!black_king_moves.contains(&Castle::Kingside.get_king_move(Side::Black)));
		assert!(!white_king_moves.contains(&Castle::Queenside.get_king_move(Side::White)));
		assert!(!black_king_moves.contains(&Castle::Queenside.get_king_move(Side::Black)));

		board.make_move(board.force_parse_move(Side::White, "Bc4"));
		board.make_move(board.force_parse_move(Side::Black, "g7"));

		let white_king_moves = board.get_legal_moves(white_king_square);
		let black_king_moves = board.get_legal_moves(black_king_square);

		assert!(white_king_moves.contains(&board.force_parse_move(Side::White, "O-O")));
		assert!(black_king_moves.contains(&board.force_parse_move(Side::Black, "O-O")));

		assert!(&board.try_parse_move(Side::White, "O-O-O").is_err());
		assert!(&board.try_parse_move(Side::Black, "O-O-O").is_err());
		assert!(!white_king_moves.contains(&Castle::Queenside.get_king_move(Side::White)));
		assert!(!black_king_moves.contains(&Castle::Queenside.get_king_move(Side::Black)));
		
		board.make_move(board.force_parse_move(Side::White, "O-O"));

		assert!((board.get_side_pieces_bitboard(Side::White, Piece::King) & Bitboard::square(Square::from_string("g1"))).has_pieces());
		assert!((board.get_side_pieces_bitboard(Side::White, Piece::Rook) & Bitboard::square(Square::from_string("f1"))).has_pieces());
		
		board.make_move(board.force_parse_move(Side::Black, "O-O"));

		assert!((board.get_side_pieces_bitboard(Side::Black, Piece::King) & Bitboard::square(Square::from_string("g8"))).has_pieces());
		assert!((board.get_side_pieces_bitboard(Side::Black, Piece::Rook) & Bitboard::square(Square::from_string("f8"))).has_pieces());

	}

	#[test]
	fn board_detects_checks() {
		let mut board = Board::empty();

		assert!(!board.is_in_check(Side::White));
		assert!(!board.is_in_check(Side::Black));

		board.add(Side::White, Piece::King, Square::from_string("e1"));
		board.add(Side::Black, Piece::King, Square::from_string("e8"));

		board.add(Side::White, Piece::Pawn, Square::from_string("e7"));
		assert!(!board.is_in_check(Side::White));
		assert!(!board.is_in_check(Side::Black));

		board.add(Side::Black, Piece::Pawn, Square::from_string("d2"));
		assert!(board.is_in_check(Side::White));
		assert!(!board.is_in_check(Side::Black));

		let mut board = Board::starting_position();
		assert!(!board.is_in_check(Side::White));
		assert!(!board.is_in_check(Side::Black));

		board.transform(Move::new(Square::from_string("e2"), Square::from_string("e4")));
		assert!(!board.is_in_check(Side::White));
		assert!(!board.is_in_check(Side::Black));
	}

	#[test]
	fn board_adds() {
		let mut board = Board::empty();

		assert_eq!(board.pieces().0, 0);

		board.add(Side::White, Piece::Rook, Square::new(File::A, Rank::One));
		assert_eq!(board.white.0, 1);
		assert_eq!(board.rooks.0, 1);
		assert_eq!(board.pieces().0, 1);

		board.add(Side::White, Piece::Pawn, Square::new(File::A, Rank::Two));
		assert_eq!(board.white.0, 1 + (1 << 8));
		assert_eq!(board.rooks.0, 1);
		assert_eq!(board.pawns.0, (1 << 8));
		assert_eq!(board.pieces().0, 1 + (1 << 8));
	}

	#[test]
	fn board_transforms() {
		let a8 = Square::from_string("a8");
		let h1 = Square::from_string("h1");
		let h2 = Square::from_string("h2");

		let mut board = Board::empty();
		board.add(Side::White, Piece::Pawn, a8);
		board.transform(Move::new(a8, h1));

		assert_eq!(board.pieces(), Bitboard::square(h1));
		assert_eq!(board.black, Bitboard::empty());
		assert_eq!(board.white, Bitboard::square(h1));
		assert_eq!(board.pawns, Bitboard::square(h1));
		assert_eq!(board.queens, Bitboard::empty());

		let new_board = board.get_transformation(Move::new(h1, h2));
		assert_eq!(new_board.pieces(), Bitboard::square(h2));
		assert_eq!(new_board.black, Bitboard::empty());
		assert_eq!(new_board.white, Bitboard::square(h2));
		assert_eq!(new_board.pawns, Bitboard::square(h2));
		assert_eq!(new_board.queens, Bitboard::empty());

		assert_eq!(board.pieces(), Bitboard::square(h1));
		assert_eq!(board.black, Bitboard::empty());
		assert_eq!(board.white, Bitboard::square(h1));
		assert_eq!(board.pawns, Bitboard::square(h1));
		assert_eq!(board.queens, Bitboard::empty());
	}

	#[test]
	fn board_vision() {
		let mut board = Board::empty();
		board.add(Side::Black, Piece::King, Square::from_string("b3"));
		board.add(Side::Black, Piece::Bishop, Square::from_string("h8"));
		board.add(Side::Black, Piece::Pawn, Square::from_string("f7"));
		board.add(Side::White, Piece::Pawn, Square::from_string("g7"));
		board.add(Side::White, Piece::Pawn, Square::from_string("f6"));

		let vision = board.get_adjacent_vision(Square::from_string("b3"));
		assert_eq!(vision.to_squares().len(), 8);

		let vision = board.get_adjacent_vision(Square::from_string("h8"));
		assert_eq!(vision.to_squares().len(), 3);

		let vision = board.get_adjacent_vision(Square::from_string("f7"));
		assert_eq!(vision.to_squares().len(), 8);


		let vision = board.get_immediately_diagonal_vision(Square::from_string("b3"));
		assert_eq!(vision.to_squares().len(), 4);

		let vision = board.get_immediately_diagonal_vision(Square::from_string("h8"));
		assert_eq!(vision.to_squares().len(), 1);

		let vision = board.get_immediately_diagonal_vision(Square::from_string("f7"));
		assert_eq!(vision.to_squares().len(), 4);


		let vision = board.get_lateral_vision(Square::from_string("b3"));
		assert_eq!(vision.to_squares().len(), 14);

		let vision = board.get_lateral_vision(Square::from_string("h8"));
		assert_eq!(vision.to_squares().len(), 14);

		let vision = board.get_lateral_vision(Square::from_string("f7"));
		assert_eq!(vision.to_squares().len(), 8);
		
	}

	#[test]
	fn board_detects_occupied_squares() {
		let mut board = Board::empty();

		board.add(Side::White, Piece::Pawn, Square::new(File::E, Rank::Four));
		board.add(Side::White, Piece::Pawn, Square::new(File::D, Rank::Four));
		board.add(Side::White, Piece::Knight, Square::from_string("g7"));
		assert!(board.is_occupied(Square::new(File::E, Rank::Four)));
		assert!(board.is_occupied(Square::new(File::D, Rank::Four)));
		assert!(!board.is_occupied(Square::new(File::D, Rank::Five)));
		assert!(!board.is_occupied(Square::new(File::C, Rank::One)));
		assert!(!board.is_occupied(Square::new(File::H, Rank::Eight)));
		assert!(board.is_occupied(Square::new(File::G, Rank::Seven)));
	}

	#[test]
	fn bitboard_instantiates_square() {
		assert_eq!(Bitboard::square(Square::new(File::A, Rank::One)).0, 1);
		assert_eq!(Bitboard::square(Square::new(File::B, Rank::One)).0, 2);
		assert_eq!(Bitboard::square(Square::new(File::A, Rank::Two)).0, (1 << 8));
		assert_eq!(Bitboard::square(Square::new(File::B, Rank::Two)).0, (1 << 9));
		assert_eq!(Bitboard::square(Square::new(File::H, Rank::Eight)).0, (1 << 63));
	}

	#[test]
	fn bitboard_instantiates_rank() {
		let expected_rank_one = 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128;
		assert_eq!(Bitboard::rank(Rank::One).0, expected_rank_one);
		assert_eq!(Bitboard::rank(Rank::Two).0, expected_rank_one << 8);
		assert_eq!(Bitboard::rank(Rank::Eight).0, expected_rank_one << 56);
	}

	#[test]
	fn bitboard_transforms() {
		let a8 = Square::from_string("a8");
		let h1 = Square::from_string("h1");

		let bitboard = Bitboard::square(a8);
		let bitboard = bitboard.transform(Move::new(a8, h1));

		assert_eq!(bitboard, Bitboard::square(h1));
	}

	#[test]
	fn bitboard_to_squares() {
		let squares = vec![Square::from_string("b7"), Square::from_string("h2")];
		let bitboard = Bitboard::from_squares(squares);

		let squares = bitboard.to_squares();
		assert_eq!(squares.len(), 2);
		assert!(squares.contains(&Square::from_string("b7")));
		assert!(squares.contains(&Square::from_string("h2")));
	}

	#[test]
	fn square_instantiates_from_string() {
		let a8 = Square::from_string("a8");
		let f6 = Square::from_string("f6");
		let h1 = Square::from_string("h1");
		assert_eq!(a8, Square(File::A, Rank::Eight));
		assert_eq!(f6, Square(File::F, Rank::Six));
		assert_eq!(h1, Square(File::H, Rank::One));
	}

	#[test]
	fn square_gets_adjacent() {
		let a8 = Square::from_string("a8");
		let f6 = Square::from_string("f6");
		let h1 = Square::from_string("h1");
		let d8 = Square::from_string("d8");

		assert_eq!(a8.get_adjacent(Direction::Left), None);
		assert_eq!(a8.get_adjacent(Direction::Right), Some(Square(File::B, Rank::Eight)));
		assert_eq!(h1.get_adjacent(Direction::Right), None);
		assert_eq!(d8.get_adjacent(Direction::UpLeft), None);
	}
}
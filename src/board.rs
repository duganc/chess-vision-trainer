use std::ops::BitAnd;
use std::ops::BitOr;

pub struct Board {
	white: Bitboard,
	black: Bitboard,
	pawns: Bitboard,
	knights: Bitboard,
	bishops: Bitboard,
	rooks: Bitboard,
	queens: Bitboard,
	kings: Bitboard
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
			kings
		}
	}

	pub fn add(&self, side: Side, piece: Piece, square: Square) {
		if self.is_occupied(square) {
			panic!("{:?} is already occupied!", square);
		}


	}

	pub fn is_occupied(&self, square: Square) -> bool {
		(Bitboard::square(square) & self.pieces()).0 > 0
	}

	pub fn get_legal_moves(&self, square: Square) -> List<Board> {
		let (side, piece) = match self.get(square) {
			Some(p) => p,
			_ => panic!("There's no piece on {:?}!", square);
		};

		let to_return = match piece {
			Piece::Pawn {
				self.get_en_passant_takes(side, square) | self.get_diagonal_takes(side, square) | self.get_forward_steps(side, square)
			},
			Piece::Knight {
				self.get_knight_moves(side, square)
			}
			Piece::Bishop {
				self.get_diagonal_moves(side, square)
			}
			Piece::Rook {
				self.get_lateral_moves(side, square)
			}
			Piece::Queen {
				self.get_diagonal_moves(side, square) | self.get_lateral_moves(side, square)
			}
			Piece::King {
				self.get_adjacent_moves(side, square)
			}
		};

		return to_return & self.get_moves_into_check().get_inverse()
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
		while !file.is_none() && !rank.is_none() {
			let (file, rank) = match direction {
				Direction::Up => (file, rank.unwrap().next()),
				Direction::Down => (file, rank.unwrap().previous()),
				Direction::Left => (file.unwrap().previous(), rank),
				Direction::Right => (file.unwrap().next(), rank),
				Direction::UpLeft => (file.unwrap().previous(), rank.unwrap().next()),
				Direction::UpRight => (file.unwrap().next(), rank.unwrap().next()),
				Direction::DownLeft => (file.unwrap().previous(), rank.unwrap().previous()),
				Direction::DownRight => (file.unwrap().previous(), rank.unwrap().next()),
			};
			let test_square = Square(file.unwrap(), rank.unwrap());
			to_return = to_return | Bitboard::square(test_square);
			if self.is_occupied(test_square) {
				break;
			}
		}

		return to_return;
	}


	pub fn get(&self, square: Square) -> Option<(Side, Piece)> {
		match self.is_occupied(square) {
			False => None,
			True => {
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

#[derive(Debug)]
pub enum Side {
	White,
	Black
}

#[derive(Debug)]
pub enum Piece {
	Pawn,
	Knight,
	Bishop,
	Rook,
	Queen,
	King
}

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
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

	pub fn previous(&self) -> Option<Self> {
		match *self {
			File::A => None,
			f => Some(File::from_u8((f as u8) - 1))
		}
	}

	pub fn next(&self) -> Option<Self> {
		match *self {
			File::H => None,
			f => Some(File::from_u8((f as u8) + 1))
		}
	}

	pub fn from_u8(i: u8) -> Self {
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

#[derive(Debug, PartialEq)]
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

	pub fn previous(&self) -> Option<Self> {
		match *self {
			Rank::One => None,
			r => Some(Rank::from_u8((r as u8) - 1))
		}
	}

	pub fn next(&self) -> Option<Self> {
		match *self {
			Rank::Eight => None,
			r => Some(Rank::from_u8((r as u8) + 1))
		}
	}

	pub fn from_u8(i: u8) -> Self {
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
}

#[derive(Debug)]
pub struct Bitboard(u8);

impl Bitboard {

	pub fn empty() -> Self {
		Bitboard(0)
	}

	pub fn is_empty(&self) -> bool {
		self.0 == 0
	}

	pub fn has_pieces(&self) -> bool {
		!self.is_empty()
	}

	pub fn rank(rank: Rank) -> Self {
		Bitboard(72340172838076673 << 2_u8.pow(64)*(rank as u8)) // 72340172838076673 = 1+2**8+2**16+2**24+2**32+2**40+2**48+2**56
	}

	pub fn file(file: File) -> Self {
		Bitboard(255 << (file as u8)) // 255 = 1+2+4+8+16+32+64+128 = 2**8 - 1
	}

	pub fn square(square: Square) -> Self {
		Self::file(square.0) & Self::rank(square.1)
	}

	pub fn get_inverse(&self) -> Self {
		Bitboard(!self.0)
	}

	// pub fn print(&self) -> String {
		
	// 	+---+---+---+---+---+---+---+---+
	// 	| R | N | B | K | Q | B | N | R |
	// 	+---+---+---+---+---+---+---+---+
	// 	| P | P | P | P | P | P | P | P |
	// 	+---+---+---+---+---+---+---+---+
	// }

}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Bitboard(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn board_smoke_test() {
		let board = Board::empty();

		board.add(Side::White, Piece::Pawn, Square::new(File::E, Rank::Four));
		board.add(Side::Black, Piece::Pawn, Square::from_string("e5"));
		board.add(Side::White, Piece::King, Square::from_string("e1"));
		board.add(Side::Black, Piece::King, Square::from_string("e8"));
		board.add(Side::White, Piece::Knight, Square::from_string("g7"));

		assert!(board.is_legal_move(Side::White, Move::new(Square::from_string("g7"), Square::from_string("h5"))));
		assert!(board.is_legal_move(Side::White, Move::from_string("Ke2")));
		assert!(!board.is_legal_move(Side::White, Move::from_string("Ka1")));
		assert!(!board.is_legal_move(Side::White, Move::from_string("e5")));
		assert!(board.is_in_check(Side::Black));
		assert!(board.is_attacking(Square::from_string("g7"), Square::from_string("e5")));
	}

	#[test]
	fn board_detects_occupied_squares() {
		let board = Board::empty();

		board.add(Side::White, Piece::Pawn, Square::new(File::E, Rank::Four));
		board.add(Side::White, Piece::Pawn, Square::new(File::D, Rank::Four));
		assert!(board.is_occupied(Square::new(File::E, Rank::Four)))
		assert!(board.is_occupied(Square::new(File::D, Rank::Four)))
		assert!(!board.is_occupied(Square::new(File::D, Rank::Five)))
		assert!(!board.is_occupied(Square::new(File::C, Rank::One)))
		assert!(!board.is_occupied(Square::new(File::H, Rank::Eight)))
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
}
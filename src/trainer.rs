
use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;
use text_io::read;
use crate::board::{Board, Move, Square, File, Rank, Side, Piece};
use crate::game::{Game};
use crate::color::Color;

const DEFAULT_N_ROUNDS: usize = 20;
const N_ROUNDS_BEFORE_SEQUENTIAL: usize = 3;
const DEFAULT_N_PIECES: usize = 3;

pub struct Trainer {
	requests: Vec<TrainerRequest>,
	state: TrainerState,
	input_source: TrainerInputSource,
	output: TrainerOutput,
	blindfold: bool,
	whites_perspective_only: bool,
	game: Game
}

impl Trainer {
	
	pub fn builder(mode: TrainerMode) -> TrainerBuilder {
		TrainerBuilder {
			mode,
			input_source: TrainerInputSource::StdIn,
			output: TrainerOutput::StdOut,
			blindfold: false,
			whites_perspective_only: false,
			game: Game::new()
		}
	}

	pub fn run(&mut self) {
		assert_eq!(self.state, TrainerState::ReadyToRun);
		while self.state != TrainerState::Finished {
			match self.state {
				TrainerState::ReadyToRun => {
					self.state = TrainerState::Running;
				},
				TrainerState::Running => {
					self.transform();
					self.prompt();
					self.state = TrainerState::WaitingForInput;
				},
				TrainerState::WaitingForInput => {
					let input = self.get_input();
					match self.validate(input) {
						Ok(_result) => {
							match self.evaluate() {
								Ok(_evaluation) => {
									self.emit(Trainer::get_success("Correct!".to_string()).unwrap());
								},
								Err(evaluation) => {
									self.emit(evaluation);
									self.state = TrainerState::Finished;
								}
							};
							if !(self.state == TrainerState::Finished) {
								if self.out_of_prompts() {
									self.state = TrainerState::Finished;
								} else {
									self.state = TrainerState::Running;
								}
							}
						},
						Err(e) => {
							self.emit(format!("Input error: {}", e));
						}
					}
				},
				TrainerState::Finished => {
					panic!("Hit Finished state!");
				}
			};
		}
	}

	fn out_of_prompts(&self) -> bool {
		self.requests.iter().filter(|x| x.get_response().is_none()).count() == 0
	}

	fn transform(&mut self) {
		let request = self.requests.iter_mut().filter(|x| x.get_response().is_none()).map(|x| x).nth(0);
		match request {
			Some(r) => {
				r.transform(&mut self.game)
			},
			None => panic!("There are no more requests!")
		};
	}

	fn validate(&mut self, input: String) -> Result<String, String> {
		let request = self.requests.iter_mut().filter(|x| x.get_response().is_none()).map(|x| x).nth(0);
		match request {
			Some(r) => {
				r.validate(&mut self.game, input)
			},
			None => panic!("There are no more requests!")
		}
	}

	fn evaluate(&mut self) -> Result<String, String> {
		let request = self.requests.iter_mut().filter(|x| !x.get_response().is_none()).map(|x| x).last();
		match request {
			Some(r) => {
				r.evaluate(&mut self.game)
			},
			None => panic!("There are no more requests!")
		}
	}

	fn get_input(&mut self) -> String {
		self.input_source.get_input()
	}

	fn get_output(&self) -> TrainerOutput {
		self.output.clone()
	}

	fn get_state(&self) -> TrainerState {
		return self.state.clone();
	}

	fn prompt(&mut self) {
		self.emit(self.force_get_next_request().get_prompt());
	}

	fn emit(&mut self, s: String) {
		let next_to_act = self.game.get_next_to_act();
		let instantiated = s.replace("{side}", &next_to_act.colorize(next_to_act.to_string()));
		let instantiated = instantiated.replace("{moves}", &self.game.pretty_print_moves());
		let instantiated = instantiated.replace("{board}", &self.pretty_print_board());
		self.output.emit(instantiated);
	}

	fn pretty_print_board(&self) -> String {
		match self.whites_perspective_only {
			true => self.game.pretty_print_board(),
			false => self.game.pretty_print_board_from_perspective()
		}
	}

	fn update_next_request_response(&mut self, response: String) {
		let request = self.requests.iter_mut().filter(|x| x.get_response().is_none()).map(|x| x).nth(0);
		match request {
			Some(r) => r.set_response(response),
			None => panic!("There are no more requests!")
		};
	}

	fn force_get_next_request(&self) -> &TrainerRequest {
		self.get_next_request().expect("There are no more requests!")
	}

	fn get_next_request(&self) -> Option<&TrainerRequest> {
		self.requests.iter().filter(|x| x.get_response().is_none()).nth(0)
	}

	fn get_success(s: String) -> Result<String, String> {
		Ok(Color::Green.format(s))
	}

	fn get_error(s: String) -> Result<String, String> {
		Err(Color::Red.format(s))
	}

}

#[derive(Debug)]
pub struct TrainerBuilder {
	mode: TrainerMode,
	input_source: TrainerInputSource,
	output: TrainerOutput,
	blindfold: bool,
	whites_perspective_only: bool,
	game: Game,
}

impl TrainerBuilder {

	pub fn with_moves(mut self, moves: String) -> Self {
		self.game.make_moves_from_string(moves);
		return self;
	}

	pub fn with_input_source(mut self, input_source: TrainerInputSource) -> Self {
		self.input_source = input_source;
		return self;
	}

	pub fn with_buffer_output(mut self) -> Self {
		self.output = TrainerOutput::Buffer(Vec::new());
		return self;
	}

	pub fn blindfold(mut self) -> Self {
		self.blindfold = true;
		return self;
	}

	pub fn whites_perspective_only(mut self) -> Self {
		self.whites_perspective_only = true;
		return self;
	}

	pub fn build(self) -> Trainer {
		Trainer {
			requests: self.get_requests(self.mode),
			state: TrainerState::ReadyToRun,
			input_source: self.input_source,
			output: self.output,
			blindfold: self.blindfold,
			whites_perspective_only: self.whites_perspective_only,
			game: self.game,
		}
	}

	fn get_requests(&self, mode: TrainerMode) -> Vec<TrainerRequest> {
		let maybe_board = if self.blindfold { "".to_string() } else { "{board}".to_string() };
		match mode {
			TrainerMode::Checks => {
				vec![
					TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&"Identify all of the checks in this position: \n".to_string() +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMovesAndEndOnRandomSide,
						TrainerResponseValidator::ListOfMovesFromCurrentPosition,
						TrainerResponseEvaluator::AreAllChecksInPosition
					)
				]
			},
			TrainerMode::Captures => {
				vec![
					TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&"Identify all of the captures in this position: \n".to_string() +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMovesAndEndOnRandomSide,
						TrainerResponseValidator::ListOfMovesFromCurrentPosition,
						TrainerResponseEvaluator::AreAllCapturesInPosition
					)
				]
			},
			TrainerMode::Sequential => {
				let mut to_return = vec![
					TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&"Identify all of the checks in this position: \n".to_string() +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMoves(2*N_ROUNDS_BEFORE_SEQUENTIAL),
						TrainerResponseValidator::ListOfMovesFromCurrentPosition,
						TrainerResponseEvaluator::AreAllChecksInPosition
					)
				];
				for _i in 0..(2*(DEFAULT_N_ROUNDS - N_ROUNDS_BEFORE_SEQUENTIAL)) {
					let request = TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&"Identify all of the checks in this position: \n".to_string() +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMove,
						TrainerResponseValidator::ListOfMovesFromCurrentPosition,
						TrainerResponseEvaluator::AreAllChecksInPosition
					);
					to_return.push(request);
				}
				return to_return;
			},
			TrainerMode::Position => {
				let mut pieces = Piece::all();
				let first = pieces.pop().unwrap();
				let mut to_return = vec![
					TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&format!("Where are all pieces of type {:?}: \n", first) +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMovesAndEndOnRandomSide,
						TrainerResponseValidator::ListOfSquares,
						TrainerResponseEvaluator::AreAllPiecePositions(first)
					)
				];

				for piece in pieces.into_iter().rev() {
					let request = TrainerRequest::new(
						"You're playing the {side} pieces.\n".to_string() +
						&format!("Where are all pieces of type {:?}: \n", piece) +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::DoNothing,
						TrainerResponseValidator::ListOfSquares,
						TrainerResponseEvaluator::AreAllPiecePositions(piece)
					);
					to_return.push(request);
				}
				return to_return;
			},
			TrainerMode::MostDefended(target) => {
				let target_string = target.to_plural_string();
				let validator = match target {
					Target::Piece => TrainerResponseValidator::ListOfPiecesForNextToAct,
					Target::Square => TrainerResponseValidator::ListOfSquares,
				};
				vec![
					TrainerRequest::new(
						format!("Identify the top {} most defended {} for {{side}}: \n", DEFAULT_N_PIECES, target_string) +
						&"{moves}\n".to_string() +
						&maybe_board,
						TrainerResponseTransformer::MakeRandomMovesAndEndOnRandomSide,
						validator,
						TrainerResponseEvaluator::AreNMostDefendedForNextToAct(DEFAULT_N_PIECES, target)
					)
				]
			}
		}
	}
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TrainerMode {
	Checks,
	Captures,
	Sequential,
	Position,
	MostDefended(Target),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Target {
	Square,
	Piece
}

impl Target {
	pub fn to_plural_string(&self) -> String {
		match self {
			Target::Piece => "pieces",
			Target::Square => "squares"
		}.to_string()
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum TrainerState {
	ReadyToRun,
	Running,
	WaitingForInput,
	Finished
}

#[derive(Debug, PartialEq, Clone)]
pub enum TrainerInputSource {
	Buffer(Vec<String>),
	StdIn
}

impl TrainerInputSource {
	fn get_input(&mut self) -> String {
		match self {
			TrainerInputSource::Buffer(buffer) => {
				match buffer.len() {
					0 => panic!("Buffer is empty!"),
					_ => {
						let to_return = buffer.clone().remove(buffer.len() - 1); // This copy is inefficient, but makes the implementation simpler
						*self = TrainerInputSource::Buffer(buffer.to_vec());
						return to_return
					}
				}
			},
			TrainerInputSource::StdIn => {
				let response: String = read!();
				return response;
			}
		}
	}
}

struct TrainerRequest {
	prompt: String,
	transformer: TrainerResponseTransformer,
	validator: TrainerResponseValidator,
	evaluator: TrainerResponseEvaluator,
	response: Option<String>
}

impl TrainerRequest {

	fn new(prompt: String, transformer: TrainerResponseTransformer, validator: TrainerResponseValidator, evaluator: TrainerResponseEvaluator) -> Self {
		Self {
			prompt,
			transformer,
			validator,
			evaluator,
			response: None
		}
	}

	fn get_prompt(&self) -> String {
		self.prompt.clone()
	}

	fn get_response(&self) -> Option<String> {
		self.response.clone()
	}

	fn set_response(&mut self, response: String) {
		self.response = Some(response);
	}

	fn transform(&mut self, game: &mut Game) {
		self.transformer.transform(game)
	}

	fn validate(&mut self, game: &Game, input: String) -> Result<String, String> {
		match self.validator.validate(game, input.clone()) {
			Ok(response) => {
				self.response = Some(input.clone());
				return Ok(response);
			},
			Err(e) => {
				return Err(e)
			}
		}

	}


	fn evaluate(&mut self, game: &Game) -> Result<String, String> {
		match &self.response {
			Some(response) => {
				return self.evaluator.evaluate(game, response.to_string());
			},
			None => panic!("Evaluate called without a validated response!")
		}
	}

}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainerResponseTransformer {
	DoNothing,
	MakeRandomMove,
	MakeRandomMoves(usize),
	MakeRandomMovesAndEndOnRandomSide,
}

impl TrainerResponseTransformer {

	fn transform(&self, game: &mut Game) {
		match self {
			Self::DoNothing => {},
			Self::MakeRandomMove => {
				game.make_random_move();
			},
			Self::MakeRandomMoves(n) => {
				game.make_random_moves(*n);
			},
			Self::MakeRandomMovesAndEndOnRandomSide => {
				if game.get_moves().len() == 0 {
					game.make_random_moves_and_end_on_random_side(DEFAULT_N_ROUNDS);
				}
			},
		}
	}

}


#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainerResponseValidator {
	ListOfSquares,
	ListOfSequentialMoves,
	ListOfMovesFromCurrentPosition,
	ListOfPiecesForNextToAct,
}

impl TrainerResponseValidator {

	fn validate(&self, game: &Game, input: String) -> Result<String, String> {
		match self {
			Self::ListOfSquares => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is a valid list of sequential squares!", input.clone()))
				}
				match Square::squares_from_string(input.clone()) {
					Ok(_squares) => return Ok(format!("{} is a valid list of squares!", input.clone())),
					Err(e) => Trainer::get_error(e)
				}
			},
			Self::ListOfSequentialMoves => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is an empty list of moves!", input.clone()));
				}
				match game.parse_sequential_moves(input.clone()) {
					Ok(_) => Ok(format!("{} is a valid list of sequential moves!", input.clone())),
					Err(e) => Trainer::get_error(e)
				}
			},
			Self::ListOfMovesFromCurrentPosition => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is an empty list of moves!", input.clone()));
				}
				match game.parse_moves_from_current_position(input.clone()) {
					Ok(_) => Ok(format!("{} is a valid list of moves from current position!", input.clone())),
					Err(e) => Trainer::get_error(e)
				}
			},
			Self::ListOfPiecesForNextToAct => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is an empty list of pieces!", input.clone()));
				}
				match game.parse_pieces_for_next_to_act(input.clone()) {
					Ok(_) => Ok(format!("{} is a valid list of pieces for next to act!", input.clone())),
					Err(e) => Trainer::get_error(e)
				}
			}
		}
	}

}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainerResponseEvaluator {
	AreAllChecksInPosition,
	AreAllCapturesInPosition,
	AreAllPiecePositions(Piece),
	AreNMostDefendedForNextToAct(usize, Target)
}

impl TrainerResponseEvaluator {

	fn evaluate(&self, game: &Game, response: String) -> Result<String, String> {
		match self {
			Self::AreAllChecksInPosition => {
				let potential_checks_result = Self::parse_moves_from_current_position(game, response);
				match potential_checks_result {
					Err(e) => {return Trainer::get_error(e);},
					Ok(checks) => {
						let potential_checks: HashSet<Move> = checks.into_iter().collect();
						let actual_checks: HashSet<Move> = game.get_checks().into_iter().collect();
						return Self::compare_move_sets(game, potential_checks, actual_checks, "checks".to_string());
						
					}
				};
			},
			Self::AreAllCapturesInPosition => {
				let potential_captures_result = Self::parse_moves_from_current_position(game, response);
				match potential_captures_result {
					Err(e) => return Trainer::get_error(e),
					Ok(captures) => {
						let potential_captures: HashSet<Move> = captures.into_iter().collect();
						let actual_captures: HashSet<Move> = game.get_captures().into_iter().collect();
						return Self::compare_move_sets(game, potential_captures, actual_captures, "captures".to_string());
						
					}
				};
			},
			Self::AreAllPiecePositions(piece) => {
				let potential_positions_result = Self::parse_squares(response);
				match potential_positions_result {
					Err(e) => return Trainer::get_error(e),
					Ok(positions) => {
						let potential_positions: HashSet<Square> = positions.into_iter().collect();
						let actual_positions: HashSet<Square> = game.get_piece_positions(*piece).into_iter().collect();
						return Self::compare_square_sets(potential_positions, actual_positions, format!("{:?} positions", piece));
					}
				}
			},
			Self::AreNMostDefendedForNextToAct(n, target) => {
				let potential_squares_result = Self::parse_squares(response);
				match potential_squares_result {
					Err(e) => return Trainer::get_error(e),
					Ok(squares) => {
						let potential_squares: HashSet<Square> = squares.into_iter().collect();

						let most_defended_squares = game.get_most_defended_squares(game.get_next_to_act());
						let most_defended = match target {
							Target::Square => most_defended_squares,
							Target::Piece => {
								let piece_squares = game.get_side_squares(game.get_next_to_act());
								most_defended_squares.into_iter().filter(|x| piece_squares.contains(&x.0)).collect()
							}
						};

						let error_string = "defended squares".to_string();
						if most_defended.len() <= *n {
							return Self::compare_square_sets(potential_squares, most_defended.into_iter().map(|x| x.0).collect(), error_string);
						} else {
							let n_value = most_defended.iter().nth(*n).unwrap().1;
							let most_defended: Vec<(Square, usize)> = most_defended.into_iter().filter(|x| x.1 >= n_value).collect();
							let actual_squares: Vec<Square> = most_defended.iter().map(|x| x.0).collect();
							let all_answers_valid = potential_squares.iter().all(|x| actual_squares.contains(&x));
							if !all_answers_valid {
								let missing = Self::get_missing_squares(potential_squares.into_iter().collect(), actual_squares);
								return Trainer::get_error(format!("Incorrect!  These aren't the most defended {}!\n{}", target.to_plural_string(), Square::squares_to_string(missing)));
							}

							// Essential are any of those strictly greater than n_value since if they're equal there could be other ones.
							let all_essential_covered = most_defended.iter().filter(|x| x.1 > n_value).all(|x| potential_squares.contains(&x.0));
							if !all_essential_covered {
								let more_defended = Self::get_missing_squares(most_defended.into_iter().filter(|x| x.1 > n_value).map(|x| x.0).collect(), potential_squares.into_iter().collect());
								return Trainer::get_error(format!("Incorrect!  There are more defended {}!\n{}", target.to_plural_string(), Square::squares_to_string(more_defended)));
							}

							if potential_squares.len() < *n {
								return Trainer::get_error(format!("Incorrect!  Only {} {} were provided but the following are all defended!\n{}", potential_squares.len(), target.to_plural_string(), Square::squares_to_string(actual_squares)));
							}

							return Trainer::get_success("Correct!".to_string());
						}
					}
				}
			}
		}
	}

	fn parse_moves_from_current_position(game: &Game, response: String) -> Result<Vec<Move>, String> {
		if response.to_lowercase() == "none" {
			Ok(Vec::new())
		} else {
			game.parse_moves_from_current_position(response)
		}
	}

	fn parse_squares(response: String) -> Result<Vec<Square>, String> {
		if response.to_lowercase() == "none" {
			Ok(Vec::new())
		} else {
			Square::squares_from_string(response)
		}
	}

	fn compare_move_sets(game: &Game, potential: HashSet<Move>, actual: HashSet<Move>, plural_name: String) -> Result<String, String> {
		let non: HashSet<Move> = potential.difference(&actual).map(|x| *x).collect();
		if non.len() > 0 {
			return Trainer::get_error(format!("Incorrect!  The following are not {}: {}", plural_name, game.get_move_strings_from_current_position(non.into_iter().collect())));
		}

		let missing: HashSet<Move> = actual.difference(&potential).map(|x| *x).collect();
		if missing.len() > 0 {
			return Trainer::get_error(format!("Incorrect!  You missed the following {}: {}", plural_name, game.get_move_strings_from_current_position(missing.into_iter().collect())));
		}

		return Trainer::get_success("Correct!".to_string());
	}

	fn compare_square_sets(potential: HashSet<Square>, actual: HashSet<Square>, plural_name: String) -> Result<String, String> {
		let non: HashSet<Square> = potential.difference(&actual).map(|x| *x).collect();
		if non.len() > 0 {
			return Trainer::get_error(format!("Incorrect!  The following are not {}: {}", plural_name, Square::squares_to_string(non.into_iter().collect())));
		}

		let missing: HashSet<Square> = actual.difference(&potential).map(|x| *x).collect();
		if missing.len() > 0 {
			return Trainer::get_error(format!("Incorrect!  You missed the following {}: {}", plural_name, Square::squares_to_string(missing.into_iter().collect())));
		}

		return Trainer::get_success("Correct!".to_string());
	}

	fn get_missing_squares(bigger: Vec<Square>, smaller: Vec<Square>) -> Vec<Square> {
		let bigger_set: HashSet<_> = bigger.into_iter().collect();
		let smaller_set: HashSet<_> = smaller.into_iter().collect();
		bigger_set.difference(&smaller_set).map(|x| *x).collect()
	}
}

#[derive(Debug, PartialEq, Clone)]
enum TrainerOutput {
	StdOut,
	Buffer(Vec<String>)
}

impl TrainerOutput {

	fn emit(&mut self, s: String) {
		match self {
			Self::StdOut => println!("{}", s),
			Self::Buffer(buffer) => buffer.push(s)
		}
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn trainer_smoke_test() {
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
		";

		let buffer = TrainerInputSource::Buffer(vec!["Bd7".to_string()]);
		let mut trainer = Trainer::builder(TrainerMode::Checks)
			.with_input_source(buffer)
			.with_buffer_output()
			.with_moves(moves.to_string())
			.build();
		trainer.run();
		
		assert_eq!(trainer.get_state(), TrainerState::Finished);
		let output = trainer.get_output();
		match output {
			TrainerOutput::Buffer(buffer) => assert_eq!(buffer[buffer.len() - 1], Color::Green.format("Correct!".to_string())),
			_ => panic!("Should have been a buffer.")
		};
		
	}

	#[test]
	fn test_builder_builds() {
		let moves = "
			Nh3,
			b5,
			Ng1".to_string();
		let buffer = TrainerInputSource::Buffer(vec!["Bd7".to_string()]);

		let trainer = Trainer::builder(TrainerMode::Checks)
			.with_input_source(buffer.clone())
			.with_buffer_output()
			.with_moves(moves.to_string())
			.blindfold()
			.whites_perspective_only()
			.build();

		assert_eq!(trainer.input_source, buffer);
		assert_eq!(trainer.output, TrainerOutput::Buffer(Vec::new()));
		let mut expected_game = Game::new();
		expected_game.make_moves_from_string(moves);
		assert_eq!(trainer.game.get_board_clone(), expected_game.get_board_clone());
		assert!(trainer.blindfold);
		assert!(trainer.whites_perspective_only);
	}
}
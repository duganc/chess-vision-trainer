
use std::fmt::Debug;
use std::hash::Hash;
use std::collections::HashSet;
use text_io::read;
use crate::board::{Board, Move, Square, File, Rank, Side};
use crate::game::{Game};

const DEFAULT_N_ROUNDS: usize = 20;

pub struct Trainer {
	requests: Vec<TrainerRequest>,
	state: TrainerState,
	input_source: TrainerInputSource,
	output: TrainerOutput,
	game: Game
}

impl Trainer {
	
	pub fn builder(mode: TrainerMode) -> TrainerBuilder {
		TrainerBuilder {
			mode,
			input_source: TrainerInputSource::StdIn,
			output: TrainerOutput::StdOut,
			game: Game::new()
		}
	}

	pub fn run(&mut self) {
		assert_eq!(self.state, TrainerState::ReadyToRun);
		while self.state != TrainerState::Finished {
			match self.state {
				TrainerState::ReadyToRun => {
					self.state = TrainerState::PreparingGame;
				},
				TrainerState::PreparingGame => {
					self.prepare_game();
					self.state = TrainerState::Running;
				}
				TrainerState::Running => {
					self.prompt();
					self.state = TrainerState::WaitingForInput;
				},
				TrainerState::WaitingForInput => {
					let input = self.get_input();
					match self.validate(input) {
						Ok(_result) => {
							match self.evaluate() {
								Ok(_evaluation) => {
									self.emit("Correct!".to_string());
								},
								Err(evaluation) => {
									self.emit(evaluation);
								}
							};
							if self.out_of_prompts() {
								self.state = TrainerState::Finished;
							} else {
								self.state = TrainerState::Running;
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

	fn prepare_game(&mut self) {
		let request = self.requests.iter_mut().filter(|x| x.get_response().is_none()).map(|x| x).nth(0);
		match request {
			Some(r) => {
				r.prepare_game(&mut self.game)
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
		let instantiated = instantiated.replace("{board}", &self.game.pretty_print_board());
		self.output.emit(instantiated);
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

}

#[derive(Debug)]
pub struct TrainerBuilder {
	mode: TrainerMode,
	input_source: TrainerInputSource,
	output: TrainerOutput,
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

	pub fn build(self) -> Trainer {
		Trainer {
			requests: Self::get_requests(self.mode),
			state: TrainerState::ReadyToRun,
			input_source: self.input_source,
			output: self.output,
			game: self.game,
		}
	}

	fn get_requests(mode: TrainerMode) -> Vec<TrainerRequest> {
		match mode {
			TrainerMode::Checks => {
				vec![
					TrainerRequest::new(
					"You're playing the {side} pieces.\n".to_string() +
					&"Identify all of the checks in this position: \n".to_string() +
					&"{moves}\n".to_string() +
					&"{board}".to_string(),
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
					&"{board}".to_string(),
					TrainerResponseValidator::ListOfMovesFromCurrentPosition,
					TrainerResponseEvaluator::AreAllCapturesInPosition
					)
				]
			}
		}
	}
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TrainerMode {
	Checks,
	Captures
}

#[derive(Debug, PartialEq, Clone)]
pub enum TrainerState {
	ReadyToRun,
	PreparingGame,
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
	validator: TrainerResponseValidator,
	evaluator: TrainerResponseEvaluator,
	response: Option<String>
}

impl TrainerRequest {

	fn new(prompt: String, validator: TrainerResponseValidator, evaluator: TrainerResponseEvaluator) -> Self {
		Self {
			prompt,
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

	fn prepare_game(&mut self, game: &mut Game) {
		self.evaluator.prepare_game(game)
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
enum TrainerResponseValidator {
	ListOfSquares,
	ListOfSequentialMoves,
	ListOfMovesFromCurrentPosition,
}

impl TrainerResponseValidator {

	fn validate(&self, game: &Game, input: String) -> Result<String, String> {
		match self {
			Self::ListOfSquares => {
				return Err("Not yet implemented!".to_string());
			},
			Self::ListOfSequentialMoves => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is an empty list of moves!", input.clone()));
				}
				match game.parse_sequential_moves(input.clone()) {
					Ok(_) => Ok(format!("{} is a valid list of sequential moves!", input.clone())),
					Err(e) => Err(e)
				}
			},
			Self::ListOfMovesFromCurrentPosition => {
				if input.clone().to_lowercase() == "none" {
					return Ok(format!("{} is an empty list of moves!", input.clone()));
				}
				match game.parse_moves_from_current_position(input.clone()) {
					Ok(_) => Ok(format!("{} is a valid list of moves from current position!", input.clone())),
					Err(e) => Err(e)
				}
			}
		}
	}

}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainerResponseEvaluator {
	AreAllChecksInPosition,
	AreAllCapturesInPosition,
}

impl TrainerResponseEvaluator {

	fn prepare_game(&self, game: &mut Game) {
		match self {
			Self::AreAllChecksInPosition => {
				if game.get_moves().len() == 0 {
					game.make_random_moves_and_end_on_random_side(DEFAULT_N_ROUNDS);
				}
			},
			Self::AreAllCapturesInPosition => {
				if game.get_moves().len() == 0 {
					game.make_random_moves_and_end_on_random_side(DEFAULT_N_ROUNDS);
				}
			}
		}

	}


	fn evaluate(&self, game: &Game, response: String) -> Result<String, String> {
		match self {
			Self::AreAllChecksInPosition => {
				let potential_checks_result = if response.to_lowercase() == "none" {
					Ok(Vec::new())
				} else {
					game.parse_moves_from_current_position(response)
				};
				match potential_checks_result {
					Err(e) => {return Err(e);},
					Ok(checks) => {
						let potential_checks: HashSet<Move> = checks.into_iter().collect();
						let actual_checks: HashSet<Move> = game.get_checks().into_iter().collect();
						return Self::compare_move_sets(game, potential_checks, actual_checks, "checks".to_string());
						
					}
				};
			},
			Self::AreAllCapturesInPosition => {
				let potential_captures_result = if response.to_lowercase() == "none" {
					Ok(Vec::new())
				} else {
					game.parse_moves_from_current_position(response)
				};
				match potential_captures_result {
					Err(e) => return Err(e),
					Ok(captures) => {
						let potential_captures: HashSet<Move> = captures.into_iter().collect();
						let actual_captures: HashSet<Move> = game.get_captures().into_iter().collect();
						return Self::compare_move_sets(game, potential_captures, actual_captures, "captures".to_string());
						
					}
				};
			}
		}
	}

	fn compare_move_sets(game: &Game, potential: HashSet<Move>, actual: HashSet<Move>, plural_name: String) -> Result<String, String> {
		let non: HashSet<Move> = potential.difference(&actual).map(|x| *x).collect();
		if non.len() > 0 {
			return Err(format!("Incorrect!  The following are not {}: {}", plural_name, game.get_move_strings_from_current_position(non.into_iter().collect())));
		}

		let missing: HashSet<Move> = actual.difference(&potential).map(|x| *x).collect();
		if missing.len() > 0 {
			return Err(format!("Incorrect!  You missed the following {}: {}", plural_name, game.get_move_strings_from_current_position(missing.into_iter().collect())));
		}

		return Ok("Correct!".to_string());
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
			TrainerOutput::Buffer(buffer) => assert_eq!(buffer[buffer.len() - 1], "Correct!".to_string()),
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
			.build();

		assert_eq!(trainer.input_source, buffer);
		assert_eq!(trainer.output, TrainerOutput::Buffer(Vec::new()));
		let mut expected_game = Game::new();
		expected_game.make_moves_from_string(moves);
		assert_eq!(trainer.game.get_board_clone(), expected_game.get_board_clone());
	}
}
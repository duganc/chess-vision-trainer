extern crate clap;
extern crate rand;
extern crate regex;
#[macro_use] extern crate lazy_static;

mod game;
mod board;
mod trainer;
mod color;
mod evaluation;

use clap::{App, SubCommand, Arg};
use evaluation::Evaluator;
use text_io::read;
use std::collections::{HashSet};
use crate::trainer::{Trainer, TrainerMode, Target};
use crate::game::{Game};
use crate::board::{Board, Side, Move, Piece};


fn main() {

	let matches = App::new("Chess Vision Tester")
		.version("0.1.0")
		.author("Chris Dugan <c.m.s.dugan@gmail.com>")
		.about("Chess Vision Tester")
		.subcommand(
			SubCommand::with_name("checks")
				.about("Can you find all the checks in a position?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				)
		).subcommand(
			SubCommand::with_name("captures")
				.about("Can you find all of the captures in a position?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				)
		).subcommand(
			SubCommand::with_name("sequential")
				.about("Can you find all of the checks as we walk through random moves?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				)
		).subcommand(
			SubCommand::with_name("position")
				.about("Can you identify the positions of all of the pieces?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				)
		).subcommand(
			SubCommand::with_name("defended")
				.about("Can you identify the 3 most defended pieces or squares?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				).arg(
					Arg::with_name("squares")
						.short("s")
						.long("squares")
				)
		).subcommand(
			SubCommand::with_name("attacked")
				.about("Can you identify the 3 most attacked pieces or squares?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				).arg(
					Arg::with_name("squares")
						.short("s")
						.long("squares")
				)
		).subcommand(
			SubCommand::with_name("path")
				.about("Can you give the shortest path for a random piece between two random squares.")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				).arg(
					Arg::with_name("whites_perspective_only")
						.short("w")
						.long("whites-perspective-only")
				).arg(
					Arg::with_name("piece")
						.short("p")
						.long("piece")
						.takes_value(true)
				)
		).subcommand(
			SubCommand::with_name("color")
				.about("Can you identify the color of a random square?")
			).subcommand(
				SubCommand::with_name("evaluate")
					.about("Evaluate the position using Pleco")
					.arg(
						Arg::with_name("fen")
						.short("f")
						.long("fen")
						.takes_value(true)
					)
		).get_matches();

	if let Some(_matches) = matches.subcommand_matches("checks") {
		let mut builder = Trainer::builder(TrainerMode::Checks);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("captures") {
		let mut builder = Trainer::builder(TrainerMode::Captures);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("sequential") {
		let mut builder = Trainer::builder(TrainerMode::Sequential);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("position") {
		let mut builder = Trainer::builder(TrainerMode::Position);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("defended") {
		let target = match matches.is_present("squares") {
			true => Target::Square,
			false => Target::Piece,
		};
		let mut builder = Trainer::builder(TrainerMode::MostDefended(target));
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("attacked") {
		let target = match matches.is_present("squares") {
			true => Target::Square,
			false => Target::Piece,
		};
		let mut builder = Trainer::builder(TrainerMode::MostAttacked(target));
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("path") {
		let piece = match matches.value_of("piece") {
			None => Piece::get_random_non_pawn(),
			Some(p) => Piece::try_parse(p.to_string()).expect(format!("{} is not a valid piece!", p).as_str()),
		};
		let mut builder = Trainer::builder(TrainerMode::ShortestPath(piece));
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		if matches.is_present("whites-perspective-only") {
			builder = builder.whites_perspective_only();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("color") {
		let mut builder = Trainer::builder(TrainerMode::Color);
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("evaluate") {
		match matches.value_of("fen") {
			None => println!("No FEN provided!"),
			Some(fen) => {
				match Evaluator::evaluate_from_fen(fen.to_string()) {
					Ok(eval) => println!("Evaluation: {:?}", eval),
					Err(s) => println!("{}", s),
				}
			}
		}
	} else {
		panic!("Invalid subcommand");
	}

}

#[cfg(test)]
mod tests {
	use super::*;

}
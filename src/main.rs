extern crate clap;
extern crate rand;
extern crate regex;
#[macro_use] extern crate lazy_static;

mod game;
mod board;
mod trainer;

use clap::{App, SubCommand, Arg};
use text_io::read;
use std::collections::{HashSet};
use crate::trainer::{Trainer, TrainerMode};
use crate::game::{Game};
use crate::board::{Board, Side, Move};


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
				)
		).subcommand(
			SubCommand::with_name("captures")
				.about("Can you find all of the captures in a position?")
				.arg(
					Arg::with_name("blindfold")
						.short("b")
						.long("blindfold")
				)
		).get_matches();

	if let Some(_matches) = matches.subcommand_matches("checks") {
		let mut builder = Trainer::builder(TrainerMode::Checks);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("captures") {
		let mut builder = Trainer::builder(TrainerMode::Captures);
		if matches.is_present("blindfold") {
			builder = builder.blindfold();
		}
		let mut trainer = builder.build();
		trainer.run();
	} else {
		panic!("Invalid subcommand");
	}

}

#[cfg(test)]
mod tests {
	use super::*;

}
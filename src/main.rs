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
		).subcommand(
			SubCommand::with_name("captures")
				.about("Can you find all of the captures in a position?")
				.arg(
					Arg::with_name("arg")
						.short("a")
						.long("arg")
				)
		).get_matches();

	if let Some(_matches) = matches.subcommand_matches("checks") {
		let mut trainer = Trainer::builder(TrainerMode::Checks).build();
		trainer.run();
	} else if let Some(matches) = matches.subcommand_matches("captures") {
		if matches.is_present("arg") {
			panic!("Arg not defined.");
		}
		let mut trainer = Trainer::builder(TrainerMode::Captures).build();
		trainer.run();
	} else {
		panic!("Invalid subcommand");
	}

}

#[cfg(test)]
mod tests {
	use super::*;

}
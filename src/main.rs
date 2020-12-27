extern crate clap;
extern crate rand;

mod board;

use clap::{App, SubCommand, Arg};


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

	if let Some(_matches) = matches.subcommand_matches("check") {
		println!("Check!");
	} else if let Some(matches) = matches.subcommand_matches("captures") {
		if matches.is_present("arg") {
			panic!("Arg not defined.");
		}
		println!("Captures!");
	} else {
		panic!("Invalid subcommand");
	}

}

#[cfg(test)]
mod tests {
	use super::*;

}
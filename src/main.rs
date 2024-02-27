#![allow(unused)]

use clap::{arg, command, value_parser, ArgAction, Command};
use palette::{commands, Colour, ColourInfo};
use std::cmp;
use std::env;
use std::iter;

fn handle_anyhow_result<T>(res: anyhow::Result<T>) -> T {
	match res {
		Ok(t) => return t,
		Err(err) => {
			eprintln!("E: {:?}", err);
			std::process::exit(1);
		}
	}
}

mod display {
	use super::*;

	const CLEAR_SEQUENCE: &'static str = "\x1b[30;m";

	pub fn print_lines_coloured(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			let block = format!("\x1b[48;2;{};{};{}m", c.colour.r, c.colour.g, c.colour.b);
			println!("{}{}{} {}", block, "      ", CLEAR_SEQUENCE, c.hex);
		}
	}

	pub fn print_lines_uncoloured(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			println!("{}", c.hex);
		}
	}

	pub fn print_block_lines(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			let block = format!("\x1b[48;2;{};{};{}m", c.colour.r, c.colour.g, c.colour.b);
			println!("{}", block);
		}
		println!("{}", CLEAR_SEQUENCE);
	}

	pub fn print_lines_code(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			let block = format!("\x1b[48;2;{};{};{}m", c.colour.r, c.colour.g, c.colour.b);
			println!("\"{block}#{}{}\",", c.hex, CLEAR_SEQUENCE);
		}
		println!("{}", CLEAR_SEQUENCE);
	}
}

fn main() {
	let limit_arg = arg!(-l --limit <AMT> "Limit number of colours displayed")
		.required(false)
		.global(true)
		.value_parser(value_parser!(usize));

	let mut program = command!()
		.subcommand(Command::new("rand").about("Generates a randomized palette"))
		.subcommand(Command::new("wal").about("Loads colours from ~/.cache/wal/colors.json"))
		.arg(
			arg!(-l --limit <AMT> "Limit number of colours displayed")
				.required(false)
				.global(true)
				.value_parser(value_parser!(usize)),
		)
		.arg(
			arg!(-p --plain "Do not colour display")
				.required(false)
				.global(true),
		)
		.arg(
			arg!(--code "Print colours as code")
				.required(false)
				.global(true),
		)
		.arg(
			arg!(-f --lined "Print full lines of colour")
				.required(false)
				.global(true),
		);

	let help = program.render_help();

	let matches = program.get_matches();
	let mut limit = matches.get_one::<usize>("limit").cloned().unwrap_or(8);

	let colours: Vec<_> = if let Some(sub) = matches.subcommand_matches("rand") {
		handle_anyhow_result(commands::rand(limit))
	} else if let Some(sub) = matches.subcommand_matches("wal") {
		handle_anyhow_result(commands::wal())
			.into_iter()
			.take(limit)
			.collect()
	} else {
		println!("{}", help);
		std::process::exit(2);
	};

	if matches.get_flag("plain") {
		display::print_lines_uncoloured(&colours);
	} else if matches.get_flag("lined") {
		display::print_block_lines(&colours);
	} else if matches.get_flag("code") {
		display::print_lines_code(&colours);
	} else {
		display::print_lines_coloured(&colours);
	}
}

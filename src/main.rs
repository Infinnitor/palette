#![allow(unused)]
use clap::{arg, command, value_parser, ArgAction, Command};
use palette::{commands, Colour, ColourInfo};
use std::cmp;
use std::env;
use std::iter;

use cli_clipboard;

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
			println!("{}{}{} {}", c.ansi_block(), "      ", CLEAR_SEQUENCE, c.hex());
		}
	}

	pub fn print_lines_uncoloured(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			println!("{}", c.hex());
		}
	}

	pub fn print_block_lines(colours: &[ColourInfo]) {
		for c in colours.into_iter() {
			println!("{}", c.ansi_block());
		}
		println!("{}", CLEAR_SEQUENCE);
	}

	pub fn print_lines_code(colours: &[ColourInfo], coloured: bool) {
		println!("[");
		for (i, c) in colours.into_iter().enumerate() {
			let mut formatted = if coloured {
				format!("\t\"{}{}{}\",", c.ansi_block(), c.hex(), CLEAR_SEQUENCE)
			} else {
				format!("\t\"{}\",", c.hex())
			};

			if i == colours.len() - 1 {
				formatted.pop();
			}
			println!("{}", formatted);
		}
		println!("{}]", if coloured { CLEAR_SEQUENCE } else { "" });
	}

	pub fn copyable_code(colours: &[ColourInfo]) -> String{
		let mut code = String::from("gradient = [");
		for (i, c) in colours.into_iter().enumerate() {
			let mut formatted = format!("\t\"{}\",", c.hex());

			if i == colours.len() - 1 {
				formatted.pop();
			}
			code.push_str(&formatted);
		}
		code
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
		.subcommand(
			Command::new("gradient")
				.aliases(["gr"])
				.about("Generates a gradient between <start> and <end>")
				.arg(arg!([start] "The starting colour").required(true))
				.arg(arg!([end] "The ending colour").required(true)),
		)
		.subcommand(
			Command::new("gradient-rand")
				.aliases(["grand"])
				.about("Generates a gradient between two random colours"),
		)
		.subcommand(Command::new("colourize").about("Colourizes plain input from stdin"))
		.arg(
			arg!(-n --limit <AMT> "Limit number of colours displayed")
				.required(false)
				.global(true)
				.value_parser(value_parser!(usize)),
		)
		.arg(arg!(--plain "Do not colour display").required(false).global(true))
		.arg(arg!(--code "Print colours as code").required(false).global(true))
		.arg(arg!(--cpcode "Code plain code to clipboard").required(false).global(true))
		.arg(arg!(--lined "Print full lines of colour").required(false).global(true));

	let help = program.render_help();

	let matches = program.get_matches();
	let mut limit = matches.get_one::<usize>("limit").cloned().unwrap_or(8);

	let colours: Vec<_> = if let Some(sub) = matches.subcommand_matches("rand") {
		handle_anyhow_result(commands::rand(limit))
	} else if let Some(sub) = matches.subcommand_matches("wal") {
		handle_anyhow_result(commands::wal()).into_iter().take(limit).collect()
	} else if let Some(sub) = matches.subcommand_matches("gradient") {
		handle_anyhow_result(commands::gradient(
			sub.get_one::<String>("start").unwrap().to_string(),
			sub.get_one::<String>("end").unwrap().to_string(),
			limit,
		))
	} else if let Some(sub) = matches.subcommand_matches("gradient-rand") {
		let ends = handle_anyhow_result(commands::rand(2));
		handle_anyhow_result(commands::gradient(ends[0].hex(), ends[1].hex(), limit))
	} else if let Some(sub) = matches.subcommand_matches("colourize") {
		handle_anyhow_result(commands::colourize())
	} else {
		println!("{}", help);
		std::process::exit(2);
	};

	if matches.get_flag("plain") {
		if matches.get_flag("code") {
			display::print_lines_code(&colours, false);
		} else {
			display::print_lines_uncoloured(&colours);
		}
	} else if matches.get_flag("lined") {
		display::print_block_lines(&colours);
	} else if matches.get_flag("code") {
		display::print_lines_code(&colours, true);
	} else {
		display::print_lines_coloured(&colours);
	}

	if matches.get_flag("cpcode") {
		let copytext = display::copyable_code(&colours);
	}
}

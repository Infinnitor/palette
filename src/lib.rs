#![allow(unused)]
pub use rgb::RGB8 as Colour;
use std::ops::Deref;
use std::io::{self, BufRead};

// Newtype
pub struct ColourInfo(Colour);

impl ColourInfo {
	pub fn hex(&self) -> String {
		format!("#{:02x}{:02x}{:02x}", self.0.r, self.0.g, self.0.b)
	}

	pub fn colour(&self) -> &Colour {
		&self.0
	}

	pub fn ansi_block(&self) -> String {
		format!("\x1b[48;2;{};{};{}m", self.0.r, self.0.g, self.0.b)
	}
}

impl Deref for ColourInfo {
	type Target = Colour;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Convert a hexadecimal string to a Colour struct
pub fn hexadecimal_to_colour(hex: &str) -> anyhow::Result<ColourInfo> {
	use std::u8;

	let mut hex = hex.to_owned();
	if hex.starts_with("#") {
		hex.remove(0);
	}
	let binding = hex.to_lowercase().chars().collect::<Vec<_>>();

	let chunked = binding.chunks(2).map(|chunk| {
		let mut x = String::new();
		x.push(chunk[0]);
		x.push(chunk[1]);
		x
	});

	let parsed: Vec<u8> = chunked
		.filter_map(|s| u8::from_str_radix(&s, 16).ok())
		.collect();

	if parsed.len() != 3 {
		panic!("Wrong format");
	}

	Ok(ColourInfo(Colour::new(parsed[0], parsed[1], parsed[2])))
}

type CommandReturn = anyhow::Result<Vec<ColourInfo>>;

pub mod commands {
	use super::*;
	use json::JsonValue;
	use std::env;
	use std::fs;
	use std::io::{self, Read};

	pub fn wal() -> CommandReturn {
		let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
		let path = format!("{}/{}", home, ".cache/wal/colors.json");
		let contents = fs::read_to_string(path)?;
		let parsed = json::parse(&contents)?;

		let mut colours_vec = Vec::new();
		if let JsonValue::Object(obj) = parsed {
			if let Some(JsonValue::Object(colours)) = obj.get("colors") {
				for (key, value) in colours.iter() {
					if let JsonValue::Short(hex) = value {
						colours_vec.push(hexadecimal_to_colour(hex.as_str())?)
					}
				}
			}
		}

		Ok(colours_vec)
	}

	pub fn rand(n: usize) -> CommandReturn {
		use rand::Rng;
		let mut rng = rand::thread_rng();

		Ok((0..n)
			.map(|_| Colour::new(rng.gen(), rng.gen(), rng.gen()))
			.map(|c| ColourInfo(c))
			.collect())
	}

	pub fn gradient(start: String, end: String, chunks: usize) -> CommandReturn {
		let mut start = hexadecimal_to_colour(&start)?.colour().clone();
		let end = hexadecimal_to_colour(&end)?.colour().clone();

		let diffs = [
			end.r as i16 - start.r as i16,
			end.g as i16 - start.g as i16,
			end.b as i16 - start.b as i16,
		];
		let add_or_sub = |colour: &mut u8, channel| {
			if channel > 0 {
				*colour += channel as u8;
			} else if channel < 0 {
				*colour -= (channel * -1) as u8;
			}
		};

		let mut grad = Vec::new();
		grad.push(ColourInfo(start));

		let divby = chunks as i16;

		for _ in 0..(chunks - 1) {
			add_or_sub(&mut start.r, diffs[0] / divby);
			add_or_sub(&mut start.g, diffs[1] / divby);
			add_or_sub(&mut start.b, diffs[2] / divby);
			grad.push(ColourInfo(start));
		}

		Ok(grad)
	}

	pub fn colourize() -> CommandReturn {
		let mut colours = vec![];
		for line in io::stdin().lock().lines() {
			let line = line?;
			let decoded = hexadecimal_to_colour(line.trim())?;
			colours.push(decoded);
		}

		Ok(colours)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ansi_test() {
		let ansi = commands::wal().unwrap();
		assert_eq!(16, ansi.len());
	}
}

#![allow(unused)]
pub use rgb::RGB8 as Colour;

pub struct ColourInfo {
	pub colour: Colour,
	pub hex: String,
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

	Ok(ColourInfo {
		colour: Colour::new(parsed[0], parsed[1], parsed[2]),
		hex: hex.clone(),
	})
}

pub mod commands {
	use super::*;
	use json::JsonValue;
	use std::env;
	use std::fs;
	use std::io::{self, Read};

	pub fn wal() -> anyhow::Result<Vec<ColourInfo>> {
		let home = env::var("HOME").unwrap_or_else(|_| "".to_owned());
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

	pub fn rand(n: usize) -> anyhow::Result<Vec<ColourInfo>> {
		use rand::Rng;
		let mut rng = rand::thread_rng();
		let choices: Vec<_> = "1234567890abcdef".chars().collect();

		Ok((0..n)
			.map(|_| {
				(0..6)
					.map(|_| rng.gen_range(0..choices.len()))
					.map(|c| choices[c])
					.collect::<String>()
			})
			.into_iter()
			.map(|hex| hexadecimal_to_colour(&hex).unwrap())
			.collect())
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

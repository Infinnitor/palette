dev:
	cargo check

build:
	cargo build

test:
	cargo test --lib -- --show-output

install:
	cargo build --release
	sudo cp target/release/palette /usr/local/bin/

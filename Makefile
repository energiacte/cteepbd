SRCDIR:=src

test:
	#cargo test -- nocapture
	cargo test

run:
	cargo run

clippy:
	cargo +nightly clippy

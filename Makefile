SRCDIR:=src

test:
	cargo test -- nocapture

run:
	cargo run

clippy:
	cargo +nightly clippy

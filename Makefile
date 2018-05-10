SRCDIR:=src

test:
	#cargo test -- nocapture
	cargo test

run:
	cargo run

clippy:
	cargo +nightly clippy

updateclippy:
	cargo +nightly install --force clippy
	#cargo +nightly install clippy --force --git https://github.com/rust-lang-nursery/rust-clippy.git
bloat:
	cargo bloat --release -n 10
	cargo bloat --release --crates -n 10

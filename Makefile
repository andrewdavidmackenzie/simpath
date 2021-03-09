all:
	cargo build --all-features
	cargo clippy
	cargo test --all-features

all:
	cargo build --all-features
	cargo clippy --all-features
	cargo test --all-features

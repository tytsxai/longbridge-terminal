.PHONY: check-i18n check-rust check-all

check-i18n:
	python3 scripts/check_i18n_guard.py

check-rust:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings
	cargo test -- --test-threads=1

check-all: check-i18n check-rust

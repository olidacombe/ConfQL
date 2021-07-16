export RUSTFLAGS = -Z macro-backtrace

.PHONY: expand
expand:
	cargo expand --lib --tests

test:
	cargo test

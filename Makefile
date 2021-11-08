#export RUSTFLAGS = -Z macro-backtrace

.PHONY: expand readme test
expand:
	cargo expand --lib --tests

readme-%:
	cd $* && cargo readme > README.md && git add README.md

readme:
	make readme-confql
	make readme-data-resolver
	make readme-proc-macro

setup:
	cargo install cargo-readme

test:
	cargo watch -x "test -- --nocapture"

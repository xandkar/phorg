.PHONY: checks
checks:
	cargo check
	cargo test
	cargo clippy -- \
		-W clippy::pedantic \
		-W clippy::cast-possible-truncation \
		-W clippy::cast-sign-loss \
		-A clippy::single_match_else \
		-A clippy::uninlined-format-args \
		-A clippy::missing_errors_doc
	cargo fmt --check

.PHONY: clippy_nursery
clippy_nursery:
	cargo clippy -- -W clippy::nursery

.PHONY: clippy_cargo
clippy_cargo:
	cargo clippy -- -W clippy::cargo

# XXX Coverage recipes assume llvm-cov is installed:
.PHONY: coverage
coverage:
	cargo llvm-cov --lib --ignore-filename-regex 'tests\.rs'

.PHONY: coverage_html
coverage_html:
	cargo llvm-cov --lib --ignore-filename-regex 'tests\.rs' --open

.PHONY: install
install:
	cargo install --path .

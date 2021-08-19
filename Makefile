.PHONY: docs docs-open sync-virgo

build:
	cargo clippy -- -D clippy::missing_docs_in_private_items \
					-D clippy::missing_safety_doc \
					-D clippy::missing_panics_doc \
					-D clippy::missing_errors_doc
	cargo test
	cargo build

docs: build
	cargo doc --no-deps --document-private-items --all-features

docs-open:
	cargo doc --no-deps --document-private-items --all-features --open

sync-virgo:
	rsync --recursive --inplace --delete --quiet --exclude={'.git','target','web/node_modules'} . virgo:dev/cachem

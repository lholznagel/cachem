.PHONY: docs docs-open sync-virgo

docs:
	cargo doc --no-deps --document-private-items --all-features

docs-open:
	cargo doc --no-deps --document-private-items --all-features --open

sync-virgo:
	rsync --recursive --update --inplace --delete --quiet --exclude={'.git','target','web/node_modules'} . virgo:dev/cachem

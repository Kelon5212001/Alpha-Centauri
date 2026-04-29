.PHONY: rust-build rust-run rust-check

rust-check:
	cd rust-native && cargo check

rust-build:
	./rust-native/scripts/build-ubuntu.sh

rust-run:
	./rust-native/dist/ubuntu/alpha-centauri

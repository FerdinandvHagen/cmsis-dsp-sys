clear:
	clear

fmt:
	cargo fmt

build-std:
	cargo build

build-thumbv7em-none-eabihf:
	cargo build --no-default-features --features libm --target  thumbv7em-none-eabihf

all: clear fmt build-std build-thumbv7em-none-eabihf

flash: build
	cargo flash --release --chip STM32F103C8

build:
	cargo build --release --target=thumbv7m-none-eabi
	cargo size --release

test:
	cargo test --lib --target=x86_64-unknown-linux-gnu

size:
	cargo size --release

rtt:
	cargo embed --release

bloat:
	cargo bloat --release --crates

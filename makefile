clean:
	cargo clean

start:
	cargo run

build:
	cargo build

debug: build
	gdb --quiet --args ./target/debug/rustdemo

strace: build
	strace -e 'connect' -f ./target/debug/rustdemo > /dev/null

vul: build
	valgrind --leak-check=full --trace-children=yes ./target/debug/rustdemo

wasm:
	rustup target add wasm32-wasi
	rustc src/main.rs --target wasm32-wasi

.PHONY: all clean start wasm

clean:
	cargo clean

start:
	cargo run

wasm:
	rustup target add wasm32-wasi
	rustc src/main.rs --target wasm32-wasi

.PHONY: all clean start wasm

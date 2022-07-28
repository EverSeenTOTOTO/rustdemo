OUT = target/debug/rustdemo

.PHONY: clean
clean:
	cargo clean

.PHONY: start
start:
	cargo run

.PHONY: build
build:
	cargo build

.PHONY: test 
test:
	cargo test

.PHONY: debug
debug: build
	gdb --quiet --args ${OUT}

.PHONY: strace
strace: build
	strace -e 'connect' -f ${OUT} > /dev/null

.PHONY: vul
vul: build
	valgrind --leak-check=full --trace-children=yes ${OUT}

.PHONY: wasm
wasm:
	rustup target add wasm32-wasi
	rustc src/main.rs --target wasm32-wasi

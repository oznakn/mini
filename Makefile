build:
	cargo build --all

run: build
	./target/debug/mini

headers:
	cbindgen --config cbindgen.toml --crate mini-library --output headers/library.h

library_test:
	gcc -L./target/debug -lmini_library -o test/example test/example.c && ./test/example

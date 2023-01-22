build:
	cargo build --all

run: build
	./target/debug/mini example/simple.ts

run_optimize: build
	./target/debug/mini --optimize example/simple.ts

library: build
	cbindgen --config cbindgen.toml --crate mini-library --output library/example/headers/library.h
	gcc -L./target/debug -lmini_library -o library/example/hello_world library/example/hello_world.c && ./library/example/hello_world

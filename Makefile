build:
	cargo build --all
	cargo build --all --target x86_64-apple-darwin

run: build
	./target/debug/mini example/simple.ts

test: build
	./target/debug/mini --optimize example/simple.ts
	arch -x86_64 gcc -L./target/x86_64-apple-darwin/debug -lmini_library -o foo foo.o
	./foo

library: build
	cbindgen --config cbindgen.toml --crate mini-library --output library/example/headers/library.h
	gcc -L./target/debug -lmini_library -o library/example/hello_world library/example/hello_world.c && ./library/example/hello_world

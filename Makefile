build:
	cargo build

run: build
	./target/debug/mini example/simple.ts

test: build
	gcc -c -o std.o std/std.c
	./target/debug/mini --optimize example/simple.ts
	gcc -o foo std.o foo.o
	./foo

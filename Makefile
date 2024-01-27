build:
	cargo build

run: build
	./target/debug/mini example/simple.ts

test: build
	/opt/homebrew/opt/llvm/bin/clang -c -emit-llvm std/std.c
	./target/debug/mini --optimize example/simple.ts
	gcc -Wl,-ld_classic -o foo foo.o
	./foo

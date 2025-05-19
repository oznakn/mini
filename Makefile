build:
	/opt/homebrew/opt/llvm/bin/clang -c -emit-llvm std/std.c
	cargo build

run: build
	./target/debug/mini example/simple.ts

test: build
	./target/debug/mini example/simple.ts
	./bin

release:
	/opt/homebrew/opt/llvm/bin/clang -c -emit-llvm std/std.c
	cargo build --release

build:
	/opt/homebrew/opt/llvm/bin/clang -c -flto=thin std/std.c
	cargo build

run: build
	./target/debug/mini --optimize example/simple.ts

test: build
	./target/debug/mini example/simple.ts
	./foo

release:
	/opt/homebrew/opt/llvm/bin/clang -c -flto=thin std/std.c
	cargo build --release

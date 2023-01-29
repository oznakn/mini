function __builtin_print_number(n: number): void {}
function __builtin_print_newline(): void {}

function echo(n: number): number {
    __builtin_print_number(n * 2);
    __builtin_print_newline();

    return n * 2;
}

echo(5);

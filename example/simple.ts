function __builtin_print_number(): void {

}

function echo(n: number): number {
    __builtin_print_number('printf', 1, n);

    return n * 2;
}

return echo(1);

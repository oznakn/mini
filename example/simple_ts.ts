function main() {
    const echo = console.log.bind(console);

    function f(a: number, b?: string) {
        echo(a, b);
    }

    // let s = 'selam' + '1';
    // echo(s);
    f(10, 'merhaba');

    echo([55, 10, 15]);

    // echo(10 * 10);
    // echo(10 / 10.5);
    // echo(10 / 10);
    // echo(5);

    // let x;
    // echo(x);
    // x = null;
    // echo (x);
}

main();

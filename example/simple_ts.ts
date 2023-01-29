function main() {
    const echo = console.log.bind(console);

    echo([55, 10, 15]);

    // echo(10 * 10);
    // echo(10 / 10.5);
    // echo(10 / 10);
    // echo(5);

    // let x;
    // echo(x);
    // x = null;
    // echo (x);

    let obj = { a: 10, b: 'merhaba' };
    echo(obj);
    echo(typeof false);
}

main();

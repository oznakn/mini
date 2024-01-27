function main() {
    const echo = console.log.bind(console);

    let obj = { a: { a: 1, b: 2 }, b: 'merhaba' };
    let x = [4];

    echo(x[0]);
}

main();

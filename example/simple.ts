// @ts-ignore
declare function echo(...s: any[]): void;

function f(a: number, b?: string) {
    echo(b);
    return a * 2;
}

// const echo = console.log.bind(console);

// let s = 'selam' + '1';
// echo(s);
echo(f(10, 'merhaba'));

// echo(10 * 10);
// echo(10 / 10.5);
// echo(10 / 10);
// echo(5);

// let x;
// echo(x);
// x = null;
// echo (x);

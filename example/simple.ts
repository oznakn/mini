declare function echo(...s: any[]): void;

function test() {
    echo([55, 10, 15]);
    echo([]);
}

echo(true);
let x = false;
echo(x, false, true);

let obj = { a: 10, b: 'merhaba' };
echo(obj);
// echo(obj.a);
echo(typeof -3);
echo(!(4 < 5));
test();

declare function echo(...s: any[]): void;

echo(true);
let x = false;
echo(x, false, true);

let obj = { a: 10, b: 'merhaba' };
echo(obj);

echo(typeof -3);
echo (!(4 < 5));

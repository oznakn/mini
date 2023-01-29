// @ts-ignore
declare function str_concat(s1: string, s2: string): string;
// @ts-ignore
declare function echo_number(n: number): number;
// @ts-ignore
declare function echo_string(n: string): number;

function x(): number {
    return 10;
}

let s: string = 'selam' + '1';
echo_string(s);

let n: number = 5;
n = 10;
echo_number(n);

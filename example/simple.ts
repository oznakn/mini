declare function strlen(s: string): number;
declare function echo_number(n: number): number;
declare function echo_string(n: string): number;

let s: string = 'selam' + '1';
echo_string(s);

let n: number = 5;
n = 10;
echo_number(n);

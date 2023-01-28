function y(): number {
    return 10;
}

function x(): number {
    return y();
}

return x();

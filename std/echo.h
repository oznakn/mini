#ifndef MINI_STD_PRINT_H
#define MINI_STD_PRINT_H

#include <stdio.h>

#include "val.h"

void echo_number(int n) {
    printf("%d\n", n);
}

void echo_string(val_t *v) {
    printf("%s\n", v->str.data);
}

#endif

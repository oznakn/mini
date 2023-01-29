#ifndef MINI_STD_PRINT_H
#define MINI_STD_PRINT_H

#include <stdio.h>

#include "str.h"

void echo_number(int n) {
    printf("%d\n", n);
}

void echo_string(str_t *s) {
    printf("%s\n", s->data);
}

#endif

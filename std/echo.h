#ifndef MINI_STD_PRINT_H
#define MINI_STD_PRINT_H

#include <stdio.h>

#include "val.h"

void *echo(val_t *v) {
    if (v == NULL) {
        printf("null\n");
    } else if (v->type == VAL_INT) {
        printf("%lld\n", v->i64);
    } else if (v->type == VAL_STR) {
        printf("%s\n", v->str.data);
    } else {
        printf("RUNTIME:: echo: expected int or string, got %d", v->type);
        exit(1);
    }

    return NULL;
}

#endif

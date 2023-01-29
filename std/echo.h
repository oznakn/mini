#ifndef MINI_STD_PRINT_H
#define MINI_STD_PRINT_H

#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

#include "val.h"

#define MAX_FLOAT_LEN 256
#define MAX_FLOAT_PRECISION 6

static void* echo_float(double f64) {
    char buf[MAX_FLOAT_LEN];
    snprintf(buf, MAX_FLOAT_LEN, "%.6f", f64);

    size_t len = strlen(buf);

    buf[len - MAX_FLOAT_PRECISION - 1] = 0;

    int32_t index = MAX_FLOAT_PRECISION;
    while (index > 0) {
        if (buf[len - index] == '0') {
            buf[len - index] = 0;
            break;
        }

        index -= 1;
    }

    if (index == MAX_FLOAT_PRECISION) {
        printf("\x1B[0;33m" "%s" "\x1B[0m\n", buf);
    } else {
        printf("\x1B[0;33m" "%s.%s" "\x1B[0m\n", buf, &buf[len - MAX_FLOAT_PRECISION]);
    }

    return NULL;
}

void *echo(val_t *v) {
    if (v == NULL) {
        puts("\x1B[2m" "undefined" "\x1B[0m");
    }
    else if (v->type == VAL_NULL) {
        puts("\x1B[1m" "null" "\x1B[0m");
    }
    else if (v->type == VAL_INT) {
        printf("\x1B[0;33m" "%lld" "\x1B[0m\n", v->i64);
    }
    else if (v->type == VAL_STR) {
        printf("%s\n", v->str.data);
    }
    else if (v->type == VAL_FLOAT) {
        echo_float(v->f64);
    }
    else {
        printf("RUNTIME:: echo: expected, got %d\n", v->type);
        exit(1);
    }

    return NULL;
}

#endif

#ifndef MINI_STD_PRINT_H
#define MINI_STD_PRINT_H

#include <assert.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

#include "val.h"

#define MAX_FLOAT_LEN 256
#define MAX_FLOAT_PRECISION 6

static void echo_internal(val_t *v);

static void echo_float(double f64) {
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
}

static void echo_array(array_t *items) {
    printf("[");

    for (uint64_t i = 0; i < items->len; i++) {
        val_t *v = (val_t *) items->data[i];

        echo_internal(v);

        if (i < items->len - 1) {
            printf(", ");
        }
    }

    printf("]");
}

static void echo_internal(val_t *v) {
    if (v == NULL) {
        printf("\x1B[2m" "undefined" "\x1B[0m");
    }
    else if (v->type == VAL_NULL) {
        printf("\x1B[1m" "null" "\x1B[0m");
    }
    else if (v->type == VAL_INT) {
        printf("\x1B[0;33m" "%lld" "\x1B[0m", v->i64);
    }
    else if (v->type == VAL_STR) {
        printf("%s", v->str.data);
    }
    else if (v->type == VAL_FLOAT) {
        echo_float(v->f64);
    }
    else if (v->type == VAL_ARRAY) {
        echo_array(&v->array);
    }
    else {
        DEBUG("RUNTIME:: echo: expected, got %d\n", v->type);
        exit(1);
    }
}

void *echo(val_t *items) {
    if (items->type != VAL_ARRAY) {
        DEBUG("RUNTIME:: echo: expected, got %d\n", items->type);
        exit(1);
    }

    for (uint64_t i = 0; i < items->array.len; i++) {
        val_t *v = (val_t *) items->array.data[i];

        echo_internal(v);

        if (i < items->array.len - 1) {
            printf(" ");
        }
    }

    printf("\n");

    free_val_if_ok(items);

    return NULL;
}

#endif

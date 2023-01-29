#ifndef MINI_STD_VAL_H
#define MINI_STD_VAL_H

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "str.h"

typedef enum  {
    VAL_INT,
    VAL_FLOAT,
    VAL_STR,
} val_type_t;

typedef struct {
    val_type_t type;
    union {
        int64_t i64;
        double f64;
        str_t str;
    };
} val_t;

val_t *new_val(val_type_t type) {
    val_t *result = malloc(sizeof(val_t));
    result->type = type;
    return result;
}

val_t *new_int_val(int64_t n) {
    val_t *result = new_val(VAL_INT);
    result->i64 = n;

    return result;
}

val_t *new_float_val(double f) {
    val_t *result = new_val(VAL_FLOAT);
    result->f64 = f;

    return result;
}

val_t *new_str_val(char *s) {
    val_t *result = new_val(VAL_STR);

    new_str(&result->str, s);

    return result;
}

val_t *val_op_plus(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_STR && v2->type == VAL_STR) {
        result = new_val(VAL_STR);

        str_combine(&result->str, &v1->str, &v2->str);
    }

    if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_int_val(v1->i64 + v2->i64);
    }

    free(v1);
    free(v2);

    return result;
}


#endif

#ifndef MINI_STD_VAL_H
#define MINI_STD_VAL_H

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    uint64_t len;
    char *data;
} str_t;

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

val_t *new_int(int64_t n) {
    val_t *result = malloc(sizeof(val_t));
    result->type = VAL_INT;
    result->i64 = n;

    return result;
}

val_t *new_float(double f) {
    val_t *result = malloc(sizeof(val_t));
    result->type = VAL_FLOAT;
    result->f64 = f;

    return result;
}

val_t *new_str(char *s) {
    uint64_t len = strlen(s);
    char *data = malloc(len + 1);
    memcpy(data, s, len + 1);

    val_t *result = malloc(sizeof(val_t));
    result->type = VAL_STR;
    result->str.len = len;
    result->str.data = data;

    return result;
}

val_t *str_combine(val_t *v1, val_t *v2) {
    if (v1->type != VAL_STR) {
        printf("RUNTIME:: str_combine: expected string, got %d", v1->type);
        exit(1);
    }

    if (v2->type != VAL_STR) {
        printf("RUNTIME:: str_combine: expected string, got %d", v2->type);
        exit(1);
    }

    char *data = malloc(v1->str.len + v2->str.len + 1);
    memcpy(data, v1->str.data, v1->str.len);
    memcpy(data + v1->str.len, v2->str.data, v2->str.len + 1);

    val_t *result = malloc(sizeof(val_t));
    result->type = VAL_STR;
    result->str.len = v1->str.len + v2->str.len;
    result->str.data = data;

    return result;
}


val_t *val_op_plus(val_t *v1, val_t *v2) {
    val_t *r = NULL;

    if (v1->type == VAL_STR && v2->type == VAL_STR) {
        r = str_combine(v1, v2);
    }

    if (v1->type == VAL_INT && v2->type == VAL_INT) {
        r = new_int(v1->i64 + v2->i64);
    }

    free(v1);
    free(v2);

    return r;
}


#endif

#ifndef MINI_STD_VAL_H
#define MINI_STD_VAL_H

#include <assert.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "str.h"
#include "array.h"

typedef enum  {
    VAL_NULL,
    VAL_INT,
    VAL_FLOAT,
    VAL_STR,
    VAL_ARRAY,
} val_type_t;

typedef struct {
    val_type_t type;
    int32_t ref_count;
    union {
        int64_t i64;
        double f64;
        str_t str;
        array_t array;
    };
} val_t;

static int32_t active_val_count = 0;

static val_t *new_val(val_type_t type) {
    val_t *result = malloc(sizeof(val_t));
    result->type = type;
    result->ref_count = 0;

    return result;
}

static void free_val_if_ok(val_t *val) {
    if (val != NULL && val->type != VAL_NULL && val->ref_count == 0) {
        DEBUG("GC: %p, active: %d", val, active_val_count);

        if (val->type == VAL_STR) {
            free_str(&val->str);
        }

        free(val);
    }
}

void link_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL) {
        active_val_count++;
        val->ref_count++;

        assert(active_val_count > 0);
        assert(val->ref_count > 0);
    }
}

void unlink_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL) {
        active_val_count--;
        val->ref_count--;

        assert(active_val_count >= 0);
        assert(val->ref_count >= 0);

        if (val->ref_count == 0) {
            free_val_if_ok(val);
        }
    }
}

val_t *new_null_val() {
    static val_t *null_val;
    if (null_val == NULL) {
        null_val = new_val(VAL_NULL);
    }

    return null_val;
}

val_t *new_int_val(int64_t n) {
    val_t *result = new_val(VAL_INT);
    result->i64 = n;

    DEBUG("new int: %lld, %p", result->i64, result);

    return result;
}

val_t *new_float_val(double f) {
    val_t *result = new_val(VAL_FLOAT);
    result->f64 = f;

    DEBUG("new float: %f, %p", result->f64, result);

    return result;
}

val_t *new_str_val(char *s) {
    val_t *result = new_val(VAL_STR);
    new_str(&result->str, s);

    DEBUG("new str: %s, %p", result->str.data, result);

    return result;
}

val_t *new_array_val(uint64_t len) {
    val_t *result = new_val(VAL_ARRAY);
    new_array(&result->array, len);

    DEBUG("new array: %lld, %p", result->array.len, result);

    return result;
}

static val_t *new_str_with_combine(val_t *v1, val_t *v2) {
    val_t *result = new_val(VAL_STR);
    str_combine(&result->str, &v1->str, &v2->str);

    DEBUG("new str with combine: %s, %p", result->str.data, result);

    return result;
}

val_t *val_op_add(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_STR && v2->type == VAL_STR) {
        result = new_str_with_combine(v1, v2);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_FLOAT) {
        result = new_float_val(v1->f64 + v2->f64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_FLOAT) {
        result = new_float_val((float) v1->i64 + v2->f64);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_INT) {
        result = new_float_val(v1->f64 + (float) v2->i64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_int_val(v1->i64 + v2->i64);
    }
    else {
        assert(false);
    }

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return result;
}


val_t *val_op_sub(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_FLOAT && v2->type == VAL_FLOAT) {
        result = new_float_val(v1->f64 - v2->f64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_FLOAT) {
        result = new_float_val((float) v1->i64 - v2->f64);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_INT) {
        result = new_float_val(v1->f64 - (float) v2->i64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_int_val(v1->i64 - v2->i64);
    }
    else {
        assert(false);
    }

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return result;
}

val_t *val_op_mul(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_FLOAT && v2->type == VAL_FLOAT) {
        result = new_float_val(v1->f64 * v2->f64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_FLOAT) {
        result = new_float_val((float) v1->i64 * v2->f64);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_INT) {
        result = new_float_val(v1->f64 * (float) v2->i64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_int_val(v1->i64 * v2->i64);
    }
    else {
        assert(false);
    }

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return result;
}

val_t *val_op_div(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_FLOAT && v2->type == VAL_FLOAT) {
        result = new_float_val(v1->f64 / v2->f64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_FLOAT) {
        result = new_float_val((float) v1->i64 / v2->f64);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_INT) {
        result = new_float_val(v1->f64 / (float) v2->i64);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_float_val((float) v1->i64 / (float) v2->i64);
    }
    else {
        assert(false);
    }

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return result;
}

void *val_array_push(val_t *items, val_t *v) {
    if (items->type != VAL_ARRAY) {
        assert(false);
    }

    array_push(&items->array, v);

    link_val(v);

    return NULL;
}


#endif
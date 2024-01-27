#ifndef MINI_STD_VAL_H
#define MINI_STD_VAL_H

#include <assert.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#include "defs.h"
#include "str.h"
#include "array.h"
#include "object.h"
#include "gc.h"

static val_t null_val = {VAL_NULL, 0};
static val_t true_val = {VAL_BOOL, 0, .b = true};
static val_t false_val = {VAL_BOOL, 0, .b = false};

static val_t *new_val(val_type_t type) {
    val_t *result = malloc(sizeof(val_t));
    result->type = type;
    result->ref_count = 0;

    return result;
}

val_t *new_null_val() {
    return &null_val;
}

val_t *new_bool_val(bool b) {
    return b ? &true_val : &false_val;
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

    DEBUG("new array: %zu, %p", result->array.len, result);

    return result;
}

val_t *new_object_val() {
    val_t *result = new_val(VAL_OBJECT);
    new_object(&result->object);

    DEBUG("new object, %p", result);

    return result;
}

val_t *val_get_type(val_t *v) {
    val_t *result = NULL;

    switch (v->type) {
        case VAL_BOOL:
            result = new_str_val("boolean");
            break;
        case VAL_INT:
            result = new_str_val("number");
            break;
        case VAL_FLOAT:
            result = new_str_val("number");
            break;
        case VAL_STR:
            result = new_str_val("string");
            break;
        default:
            result = new_str_val("object");
            break;
    }

    return result;
}

val_t *val_get_value(val_t *v, char *key) {
    DEBUG("val_get_value: %s", key);

    return object_get(&v->object, key);
}

#endif

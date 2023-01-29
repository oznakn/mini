#ifndef MINI_STD_OPS_H
#define MINI_STD_OPS_H

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


val_t *val_op_mod(val_t *v1, val_t *v2) {
    val_t *result = NULL;

    if (v1->type == VAL_INT && v2->type == VAL_INT) {
        result = new_int_val(v1->i64 % v2->i64);
    }
    else {
        assert(false);
    }

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return result;
}


short val_compare(val_t *v1, val_t *v2) {
    if (v1->type == VAL_FLOAT && v2->type == VAL_FLOAT) {
        return (v1->f64 < v2->f64) ? -1 : ((v1->f64 > v2->f64) ? 1 : 0);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_FLOAT) {
        return ((float) v1->i64 < v2->f64) ? -1 : (((float) v1->i64 > v2->f64) ? 1 : 0);
    }
    else if (v1->type == VAL_FLOAT && v2->type == VAL_INT) {
        return (v1->f64 < (float) v2->i64) ? -1 : ((v1->f64 > (float) v2->i64) ? 1 : 0);
    }
    else if (v1->type == VAL_INT && v2->type == VAL_INT) {
        return (v1->i64 < v2->i64) ? -1 : ((v1->i64 > v2->i64) ? 1 : 0);
    }

    assert(false);
    return 0;
}

void *val_op_eq(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status == 0);
}

void *val_op_neq(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status != 0);
}

void *val_op_seq(val_t *v1, val_t *v2) {
    if (v1 == NULL || v2 == NULL) {
        return new_bool_val(v1 == v2);
    }

    if (v1->type != v2->type) {
        return new_bool_val(false);
    }

    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status == 0);
}

void *val_op_sneq(val_t *v1, val_t *v2) {
    if (v1 == NULL || v2 == NULL) {
        return new_bool_val(v1 == v2);
    }

    if (v1->type != v2->type) {
        return new_bool_val(false);
    }

    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status != 0);
}

void *val_op_lt(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status < 0);
}

void *val_op_gt(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status > 0);
}

void *val_op_lte(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status <= 0);
}

void *val_op_gte(val_t *v1, val_t *v2) {
    short status = val_compare(v1, v2);

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(status >= 0);
}

void *val_op_and(val_t *v1, val_t *v2) {
    if (v1->type != VAL_BOOL || v2->type != VAL_BOOL) {
        assert(false);
    }

    bool result = v1->b && v2->b;

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(result);
}

void *val_op_or(val_t *v1, val_t *v2) {
    if (v1->type != VAL_BOOL || v2->type != VAL_BOOL) {
        assert(false);
    }

    bool result = v1->b || v2->b;

    free_val_if_ok(v1);
    free_val_if_ok(v2);

    return new_bool_val(result);
}

void *val_op_not(val_t *v) {
    if (v->type != VAL_BOOL) {
        assert(false);
    }

    bool result = !v->b;

    free_val_if_ok(v);

    return new_bool_val(result);
}

void *val_op_pos(val_t *v) {
    if (v->type == VAL_INT || v->type == VAL_FLOAT) {
        return v;
    }

    val_t *result;

    if (v->type == VAL_BOOL) {
        result = new_int_val(v->b ? 1 : 0);
    } else {
        assert(false);
    }

    free_val_if_ok(v);

    return result;
}

void *val_op_neg(val_t *v) {
    val_t *result;

    if (v->type == VAL_INT) {
        return new_int_val(-v->i64);
    } else if (v->type == VAL_FLOAT) {
        return new_float_val(-v->f64);
    } else {
        assert(false);
    }

    free_val_if_ok(v);

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

void *val_object_set(val_t *kv, char *k, val_t *v) {
    if (kv->type != VAL_OBJECT) {
        assert(false);
    }

    object_set(&kv->object, k, v);

    link_val(v);

    return NULL;
}

void *val_object_get(val_t *kv, char *k) {
    if (kv->type != VAL_OBJECT) {
        assert(false);
    }

    return object_get(&kv->object, k);
}

#endif

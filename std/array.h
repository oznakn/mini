#ifndef MINI_STD_ARRAY_H
#define MINI_STD_ARRAY_H

#include "defs.h"

void link_val(val_t *val);
void unlink_val(val_t *val);

static void free_array(array_t *s) {
    DEBUG("FREE ARRAY: %p", s);

    for (size_t i = 0; i < s->len; i++) {
        unlink_val((val_t *) s->data[i]);
    }

    free(s->data);
}

static void new_array(array_t *result, uint64_t capacity) {
    void **data = malloc(capacity * sizeof(void *));

    result->capacity = capacity;
    result->len = 0;
    result->data = data;
}

static void array_push(array_t *result, void *v) {
    if (result->len == result->capacity) {
        result->capacity *= 2;
        result->data = realloc(result->data, result->capacity * sizeof(void *));
    }

    result->data[result->len] = v;
    result->len++;

    DEBUG("ARRAY: push: %p, %p", result, v);

    link_val((val_t *) v);
}

#endif

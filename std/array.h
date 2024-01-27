#ifndef MINI_STD_ARRAY_H
#define MINI_STD_ARRAY_H

#include "defs.h"

void link_val(val_t *val);
void unlink_val(val_t *val);

static void free_array(array_t *s) {
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
}

static void array_insert(array_t *result, size_t index, void *v) {
    while (index >= result->capacity) {
        result->capacity *= 2;
        result->data = realloc(result->data, result->capacity * sizeof(void *));
    }

    result->data[index] = v;
    result->len = result->len > index + 1 ? result->len : index + 1;
}

static void *array_get(array_t *result, size_t index) {
    if (index >= result->len) {
        assert(false);
    }

    return result->data[index];
}

#endif

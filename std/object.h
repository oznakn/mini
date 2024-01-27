#ifndef MINI_STD_OBJECT_H
#define MINI_STD_OBJECT_H

#include "defs.h"

void link_val(val_t *val);
void unlink_val(val_t *val);

static void free_object(object_t *kv) {
    free(kv->keys);
    free(kv->vals);
}

static void new_object(object_t *result) {
    char **keys = malloc(sizeof(char *));
    void **vals = malloc(sizeof(void *));

    result->capacity = 1;
    result->len = 0;
    result->keys = keys;
    result->vals = vals;
}

static bool object_set(object_t *result, char *k, void *v) {
    for (size_t i = 0; i < result->len; i++) {
        if (strcmp(result->keys[i], k) == 0) {
            result->vals[i] = v;

            return false; // means we didn't add a new key
        }
    }

    if (result->len == result->capacity) {
        result->capacity *= 2;
        result->keys = realloc(result->keys, result->capacity * sizeof(void *));
        result->vals = realloc(result->vals, result->capacity * sizeof(void *));
    }

    result->keys[result->len] = k;
    result->vals[result->len] = v;
    result->len++;

    return true; // means we added a new key
}

static void *object_get(object_t *result, char *k) {
    for (size_t i = 0; i < result->len; i++) {
        if (strcmp(result->keys[i], k) == 0) {
            return result->vals[i];
        }
    }

    return NULL;
}


#endif

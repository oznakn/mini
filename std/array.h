#ifndef MINI_STD_ARRAY_H
#define MINI_STD_ARRAY_H

typedef struct {
    uint64_t capacity;
    uint64_t len;
    void **data;
} array_t;

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
}

#endif

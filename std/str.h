#ifndef MINI_STD_STRING_H
#define MINI_STD_STRING_H

#include <stdint.h>
#include <string.h>

typedef struct {
    uint64_t len;
    char *data;
} str_t;


str_t *new_str(char *s) {
    uint64_t len = strlen(s);
    char *data = malloc(len + 1);
    memcpy(data, s, len + 1);

    str_t *result = malloc(sizeof(str_t));
    result->len = len;
    result->data = data;

    printf("RUNTIME:: new_string: %s\n", data);

    return result;
}

uint64_t str_length(str_t *s) {
    return s->len;
}

str_t *str_combine(str_t *s1, str_t *s2) {
    char *data = malloc(s1->len + s2->len + 1);
    memcpy(data, s1->data, s1->len);
    memcpy(data + s1->len, s2->data, s2->len + 1);

    str_t *result = malloc(sizeof(str_t));
    result->len = s1->len + s2->len;
    result->data = data;

    printf("RUNTIME:: combined_string: %s\n", data);

    free(s1);
    free(s2);

    return result;
}

#endif

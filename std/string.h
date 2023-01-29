#ifndef MINI_STD_STRING_H
#define MINI_STD_STRING_H

#include <string.h>

typedef struct {
    size_t len;
    char *data;
} string_t;

string_t *new_string(const char *s) {
    size_t len = strlen(s);
    char *data = malloc(len + 1);
    memcpy(data, s, len + 1);

    string_t *result = malloc(sizeof(string_t));
    result->len = len;
    result->data = data;

    return result;
}

char *string_concat(const char *s1, const char *s2) {
    size_t len1 = strlen(s1);
    size_t len2 = strlen(s2);

    char *result = malloc(len1 + len2 + 1);
    memcpy(result, s1, len1);
    memcpy(result + len1, s2, len2 + 1);

    return result;
}

#endif

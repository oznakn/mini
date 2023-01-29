#ifndef MINI_STD_STRING_H
#define MINI_STD_STRING_H

#include <string.h>

char *str_concat(const char *s1, const char *s2) {
    size_t len1 = strlen(s1);
    size_t len2 = strlen(s2);

    char *result = malloc(len1 + len2 + 1);
    memcpy(result, s1, len1);
    memcpy(result + len1, s2, len2 + 1);

    return result;
}

char *str_move(const char *s) {
    size_t len = strlen(s);

    char *result = malloc(len + 1);
    memcpy(result, s, len + 1);

    return result;
}

#endif

#ifndef MINI_STD_STR_H
#define MINI_STD_STR_H

typedef struct {
    uint64_t len;
    char *data;
} str_t;

static void free_str(str_t *s) {
    free(s->data);
}

static void new_str(str_t *result, char *s) {
    uint64_t len = strlen(s);
    char *data = malloc(len + 1);
    memcpy(data, s, len + 1);

    result->len = len;
    result->data = data;
}

static void str_combine(str_t *result, str_t *s1, str_t *s2) {
    char *data = malloc(s1->len + s2->len + 1);
    memcpy(data, s1->data, s1->len);
    memcpy(data + s1->len, s2->data, s2->len + 1);

    result->len = s1->len + s2->len;
    result->data = data;
}

#endif

#ifndef MINI_STD_DEFS_H
#define MINI_STD_DEFS_H

typedef struct {
    uint64_t len;
    char *data;
} str_t;

typedef struct {
    size_t capacity;
    size_t len;
    void **data;
} array_t;

typedef enum  {
    VAL_NULL,
    VAL_INT,
    VAL_FLOAT,
    VAL_STR,
    VAL_ARRAY,
} val_type_t;

typedef struct {
    val_type_t type;
    int32_t ref_count;
    union {
        int64_t i64;
        double f64;
        str_t str;
        array_t array;
    };
} val_t;

#endif

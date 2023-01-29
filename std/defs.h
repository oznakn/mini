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

typedef struct {
    size_t capacity;
    size_t len;
    char **keys;
    void **vals;
} object_t;

typedef enum  {
    VAL_NULL,
    VAL_BOOL,
    VAL_INT,
    VAL_FLOAT,
    VAL_STR,
    VAL_ARRAY,
    VAL_OBJECT,
} val_type_t;

typedef struct {
    val_type_t type;
    int32_t ref_count;
    union {
        bool b;
        int64_t i64;
        double f64;
        str_t str;
        array_t array;
        object_t object;
    };
} val_t;

#endif

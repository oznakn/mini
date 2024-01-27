#ifndef MINI_STD_GC_H
#define MINI_STD_GC_H

#include "defs.h"

static int32_t active_val_count = 0;

static void free_val_if_ok(val_t *val) {
    if (val != NULL && val->type != VAL_NULL && val->type != VAL_BOOL && val->ref_count == 0) {
        DEBUG("GC: %p, type: %d", val, val->type);

        if (val->type == VAL_STR) {
            free_str(&val->str);
        } else if (val->type == VAL_ARRAY) {
            for (size_t i = 0; i < val->array.len; i++) {
                unlink_val(val->array.data[i]);
            }

            free_array(&val->array);
        } else if (val->type == VAL_OBJECT) {
            for (size_t i = 0; i < val->object.len; i++) {
                unlink_val(val->object.vals[i]);
            }

            free_object(&val->object);
        }

        free(val);
    }
}

void *link_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL && val->type != VAL_BOOL) {
        active_val_count++;
        val->ref_count++;

        assert(active_val_count > 0);
        assert(val->ref_count > 0);

        DEBUG("link: %p, type: %d, active: %d", val, val->type, active_val_count);
    }

    return NULL;
}

void *unlink_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL && val->type != VAL_BOOL) {
        active_val_count--;
        val->ref_count--;

        assert(active_val_count >= 0);
        assert(val->ref_count >= 0);

        DEBUG("unlink: %p, type: %d, active: %d", val, val->type, active_val_count);

        if (val->ref_count == 0) {
            free_val_if_ok(val);
        }
    }

    return NULL;
}

#endif

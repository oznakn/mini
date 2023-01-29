#ifndef MINI_STD_GC_H
#define MINI_STD_GC_H

#include "defs.h"

static int32_t active_val_count = 0;

static void free_val_if_ok(val_t *val) {
    if (val != NULL && val->type != VAL_NULL && val->ref_count == 0) {
        DEBUG("GC: %p, type: %d", val, val->type);

        if (val->type == VAL_STR) {
            free_str(&val->str);
        } else if (val->type == VAL_ARRAY) {
            free_array(&val->array);
        }

        free(val);
    }
}

void link_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL) {
        active_val_count++;
        val->ref_count++;

        assert(active_val_count > 0);
        assert(val->ref_count > 0);

        DEBUG("link: %p, type: %d, active: %d", val, val->type, active_val_count);
    }
}

void unlink_val(val_t *val) {
    if (val != NULL && val->type != VAL_NULL) {
        active_val_count--;
        val->ref_count--;

        assert(active_val_count >= 0);
        assert(val->ref_count >= 0);

        if (val->ref_count == 0) {
            free_val_if_ok(val);
        }

        DEBUG("unlink: %p, type: %d, active: %d", val, val->type, active_val_count);
    }
}

#endif

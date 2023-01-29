#ifndef MINI_STD_NUMBER_H
#define MINI_STD_NUMBER_H

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int value;
} Integer;

Integer* new_int(int value) {
    Integer *n = malloc(sizeof(Integer));
    n->value = value;
    return n;
}

#endif

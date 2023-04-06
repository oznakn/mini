#include <assert.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#define DEBUG_MODE false

#if DEBUG_MODE
    #define DEBUG(args...) { \
        fprintf(stderr, "   > "); \
        fprintf(stderr, ##args); \
        fprintf(stderr, "\n"); \
    };
#else
	#define DEBUG(args...) {};
#endif

#include "defs.h"
#include "val.h"
#include "ops.h"
#include "echo.h"

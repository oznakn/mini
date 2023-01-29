#include <assert.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

#define DEBUG_MODE false

#if DEBUG_MODE
    #define DEBUG(args...) { \
        fprintf(stderr, "RUNTIME:: "); \
        fprintf(stderr, ##args); \
    };
#else
	#define DEBUG(args...) {};
#endif

#include "val.h"
#include "echo.h"

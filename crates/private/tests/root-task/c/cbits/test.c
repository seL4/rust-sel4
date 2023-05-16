#include <stdlib.h>

int test(const char *s) {
    return atoi(s) + atoi(s + 1);
}

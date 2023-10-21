/*
 * Copyright 2023, Colias Group, LLC
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include <stdlib.h>

int test(const char *s) {
    return atoi(s) + atoi(s + 1);
}

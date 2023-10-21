/*
 * Copyright 2023, Colias Group, LLC
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#define BEGIN_LOCAL_FUNC(_name) \
    .type _name, %function ; \
_name:

#define BEGIN_FUNC(_name) \
    .global _name ; \
    BEGIN_LOCAL_FUNC(_name)

#define END_FUNC(_name) \
    .size _name, .-_name

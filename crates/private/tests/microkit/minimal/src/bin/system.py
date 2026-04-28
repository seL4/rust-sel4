#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

from sdfgen import SystemDescription
from ukit_test_utils import *


def f(sdf):
    sdf.add_pd(SystemDescription.ProtectionDomain(
        'test',
        'test.elf',
        stack_size=DEFAULT_STACK_SIZE,
    ))


run_script(f)

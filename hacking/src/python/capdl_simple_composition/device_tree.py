#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

from pyfdt.pyfdt import FdtBlobParse

class DeviceTree:

    def __init__(self, path):
        with open(path, 'rb') as f:
            dtb = FdtBlobParse(f)

        self._inner = dtb.to_fdt()

    def _inner(self):
        return self._inner

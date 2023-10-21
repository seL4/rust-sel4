#!/usr/bin/env python3
#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys

def check(content):
    if len(content) > 0 and content[-1] != ord('\n'):
        return False
    if len(content) > 1 and content[-2] == ord('\n'):
        return False
    return True

def main():
    ok = True
    for line in sys.stdin:
        path = line.rstrip()
        with open(path, 'rb') as f:
            content = f.read()
        if not check(content):
            print("!", path)
            ok = False
    if not ok:
        sys.exit(1)

if __name__ == '__main__':
    main()

#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys
import argparse
import pexpect

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('simulate')
    args = parser.parse_args()
    run(args)

def run(args):
    child = pexpect.spawn(args.simulate, encoding='utf-8')
    child.logfile = sys.stdout
    child.expect('banscii>', timeout=10)
    child.sendline('Hello, World!')
    child.expect('banscii>', timeout=10)
    print()

if __name__ == '__main__':
    main()

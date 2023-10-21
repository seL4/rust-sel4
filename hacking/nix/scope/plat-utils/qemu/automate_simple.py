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
    parser.add_argument('--simulate')
    parser.add_argument('--timeout', type=int)
    args = parser.parse_args()
    run(args)

def run(args):
    child = pexpect.spawn(args.simulate, encoding='utf-8')
    child.logfile = sys.stdout
    ix = child.expect(['TEST_PASS', 'TEST_FAIL', pexpect.TIMEOUT], timeout=args.timeout)
    print()
    if ix != 0:
        if ix == 1:
            sys.exit('> test reported failure')
        if ix == 2:
            sys.exit('> test timed out')
        assert False

if __name__ == '__main__':
    main()

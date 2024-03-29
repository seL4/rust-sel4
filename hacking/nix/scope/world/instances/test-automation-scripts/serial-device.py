#
# Copyright 2024, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys
import time
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
    child.expect('echo> ', timeout=5)
    time.sleep(1)
    child.send('xxx')
    child.expect('\[x\]\[x\]\[x\]', timeout=5)
    print()

if __name__ == '__main__':
    main()

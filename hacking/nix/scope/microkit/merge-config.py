#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys
import argparse
import toml

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-d')
    parser.add_argument('config', type=argparse.FileType('r'), nargs='*')
    args = parser.parse_args()
    run(args)

def run(args):
    source = {
        'crates-io': {
            'replace-with': 'vendored-sources',
        },
        'vendored-sources': {
            'directory': args.d,
        },
    }
    for f in args.config:
        obj = toml.load(f)['source']
        for (k, v) in obj.items():
            if k not in ['vendored-sources', 'crates-io']:
                source[k] = v
    toml.dump({ 'source': source }, sys.stdout)

if __name__ == '__main__':
    main()

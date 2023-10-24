#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
import difflib
import json
import subprocess
import sys
import toml
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--plan', type=Path)
    parser.add_argument('--just-check', default=False, action=argparse.BooleanOptionalAction)
    args = parser.parse_args()

    with args.plan.open() as f:
        plan = json.load(f)

    run(plan, args.just_check)


def run(plan, just_check):
    for manifest_path, v in plan.items():
        manifest_path = Path(manifest_path)
        src = Path(v['src'])
        just_check_equivalence = v['justEnsureEquivalence']

        if just_check:
            if not manifest_path.is_file():
                die(f'{manifest_path} does not exist')

        if just_check_equivalence:
            check_equivalence(manifest_path, src)
        else:
            diff_proc = subprocess.run(['diff', '-u', manifest_path, src], stdout=subprocess.PIPE)
            if diff_proc.returncode != 0:
                if just_check:
                    diff = diff_proc.stdout.decode('utf-8')
                    die(f'{manifest_path} differs from {src}:\n{diff}')
                else:
                    if subprocess.run(['cp', '-vL', '--no-preserve=all', src, manifest_path]).returncode != 0:
                        die()


def check_equivalence(a_path, b_path):
    def read(f):
      return json.dumps(toml.load(f), indent=2, sort_keys=True).splitlines()

    with a_path.open() as f:
      a = read(f)

    with b_path.open() as f:
      b = read(f)

    if a != b:
        print(f'{a_path} differs from {b_path}:', file=sys.stderr)

        for line in difflib.unified_diff(a, b, fromfile=a_path, tofile=b_path, lineterm=""):
            print(line, file=sys.stderr)

        die()


def die(msg=''):
    if msg:
        print(f'error: {msg}', file=sys.stderr)
    sys.exit(1)


if __name__ == '__main__':
    main()

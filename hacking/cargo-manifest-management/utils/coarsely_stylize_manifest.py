#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys
import json
import toml
from collections import OrderedDict


def main():
    raw = json.load(sys.stdin)
    assert isinstance(raw, dict)
    stylized = stylize([], raw)
    toml.dump(stylized, sys.stdout, encoder=toml.TomlEncoder(preserve=True))


def stylize(path, v):
    if isinstance(v, dict):
        if should_inline_table(path):
            new = InlineTableOrderedDict()
        else:
            new = OrderedDict()
        keys = list(v.keys())
        sort_table(path, keys)
        for key in keys:
            new[key] = stylize(path + [key], v[key])
        return new
    else:
        return v


class InlineTableOrderedDict(OrderedDict, toml.decoder.InlineTableDict):
    pass


def should_inline_table(path):
    if len(path) <= 1:
        return False
    if len(path) <= 3 and path[0] == 'target':
        return False
    if len(path) <= 3 and path[0] == 'profile':
        return False
    return True


# TODO: remove any bits of policy that rustfmt overrides
def sort_table(path, keys):
    if len(path) == 0:
        keys.sort(
            key=key_using_ranking(
                front=[
                    'package',
                    'lib',
                    'bin',
                    'features',
                    'dependencies',
                    'dev-dependencies',
                    'build-dependencies',
                    'workspace',
                    'profile',
                    ],
                ),
            )
    elif len(path) == 1 and path[0] == 'package':
        keys.sort(
            key=key_using_ranking(
                front=[
                    'name',
                    'version',
                    ],
                back=[
                    'description',
                    ],
                ),
            )
    elif len(path) >= 2 and path[-2].endswith('dependencies'):
        keys.sort(
            key=key_using_ranking(
                front=[
                    'path',
                    'git',
                    'branch',
                    'tag',
                    'rev',
                    'version',
                    'registry',
                    'default-features',
                    'features',
                    'optional',
                    ],
                ),
            )
    elif len(path) == 2 and path[0] == 'target':
        keys.sort(
            key=key_using_ranking(
                front=[
                    'dependencies',
                    'dev-dependencies',
                    'build-dependencies',
                    ],
                ),
            )
    elif len(path) == 1 and path[0] == 'workspace':
        keys.sort(
            key=key_using_ranking(
                back=[
                    'members',
                    ],
                ),
            )


def key_using_ranking(front=[], back=[]):
    def key(v):
        try:
            front_rank = front.index(v)
        except ValueError:
            front_rank = len(front)
        try:
            back_rank = back.index(v)
        except ValueError:
            back_rank = -1
        return (front_rank, back_rank, v)
    return key


if __name__ == '__main__':
    main()

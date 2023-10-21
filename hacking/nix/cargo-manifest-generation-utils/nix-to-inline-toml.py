#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import sys
import json
import toml
from collections import OrderedDict

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

def key_using_ranking(ranking):
    def key(v):
        try:
            return ranking.index(v) - len(ranking)
        except ValueError:
            return 0
    return key

def sort_table(path, keys):
    if len(path) == 0:
        keys.sort(key=key_using_ranking([
            'package',
            'lib',
            'bin',
            'features',
            'dependencies',
            'dev-dependencies',
            'build-dependencies',
            'workspace',
            'profile',
        ]))
    if len(path) >= 2 and path[-2].endswith('dependencies'):
        keys.sort(key=key_using_ranking([
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
        ]))
    if len(path) == 2 and path[0] == 'target':
        keys.sort(key=key_using_ranking([
            'dependencies',
            'dev-dependencies',
            'build-dependencies',
        ]))

def structure(path, v):
    if isinstance(v, dict):
        if should_inline_table(path):
            new = InlineTableOrderedDict()
        else:
            new = OrderedDict()
        keys = list(v.keys())
        sort_table(path, keys)
        for key in keys:
            new[key] = structure(path + [key], v[key])
        return new
    else:
        return v

raw = json.load(sys.stdin)

assert isinstance(raw, dict)

structured = structure([], raw)

toml.dump(structured, sys.stdout, encoder=toml.TomlEncoder(preserve=True))

#!/usr/bin/env bash
#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

set -eu -o pipefail

here=$(realpath $(dirname $0))
top_level_dir=$here/../..

find $top_level_dir/crates \
        -type f -name '*.rs' \
        -exec grep '^#!\[feature(' '{}' ';' \
    | sort \
    | uniq

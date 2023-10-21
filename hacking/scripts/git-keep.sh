#!/bin/sh
#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

# example usage: git-keep origin HEAD

set -eu

remote="${1:-origin}"
ref="${2:-HEAD}"
short_rev=$(git rev-parse --short=32 "$ref")
tag=keep/$short_rev
git tag $tag $short_rev
git push "$remote" $tag

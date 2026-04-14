#
# Copyright 2026, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
from typing import List
from dataclasses import dataclass
from sdfgen import SystemDescription

__all__ = [
    'run_script',
    'DEFAULT_STACK_SIZE',
]

DEFAULT_STACK_SIZE = 0x10_000

@dataclass
class Board:
    name: str
    arch: SystemDescription.Arch
    paddr_top: int

BOARDS: List[Board] = [
    Board(
        name='qemu_virt_aarch64',
        arch=SystemDescription.Arch.AARCH64,
        paddr_top=0x6_0000_000,
    ),
    Board(
        name='qemu_virt_riscv64',
        arch=SystemDescription.Arch.RISCV64,
        paddr_top=0x6_0000_000,
    ),
    Board(
        name='x86_64_generic',
        arch=SystemDescription.Arch.X86_64,
        paddr_top=0,
    ),
]

def run_script(f):
    parser = argparse.ArgumentParser()
    parser.add_argument('--board', required=True)
    parser.add_argument('-o', required=True, type=argparse.FileType('w'))
    args = parser.parse_args()

    board = next(filter(lambda b: b.name == args.board, BOARDS))
    sdf = SystemDescription(board.arch, board.paddr_top)
    f(sdf)

    args.o.write(sdf.render())

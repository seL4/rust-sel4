#
# Copyright 2025, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
from typing import List
from sdfgen import SystemDescription, Sddf, DeviceTree
from dataclasses import dataclass

ProtectionDomain = SystemDescription.ProtectionDomain
MemoryRegion = SystemDescription.MemoryRegion
Channel = SystemDescription.Channel
Map = SystemDescription.Map
Irq = SystemDescription.Irq


@dataclass
class Board:
    name: str
    arch: SystemDescription.Arch
    paddr_top: int
    serial_mmio_phys_addr: int
    serial_irq: int


BOARDS: List[Board] = [
    Board(
        name='qemu_virt_aarch64',
        arch=SystemDescription.Arch.AARCH64,
        paddr_top=0x6_0000_000,
        serial_mmio_phys_addr=0x9_000_000,
        serial_irq=33,
    ),
]


DEFAULT_STACK_SIZE = 0x10_000


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--board', required=True, choices=[b.name for b in BOARDS])
    parser.add_argument('-o', required=True, type=argparse.FileType('w'))
    args = parser.parse_args()
    run(args)


def run(args):
    board = next(filter(lambda b: b.name == args.board, BOARDS))
    sdf = SystemDescription(board.arch, board.paddr_top)

    serial_mmio = MemoryRegion(sdf, 'serial_mmio', 0x1_000, paddr=board.serial_mmio_phys_addr)
    assistant_to_artist = MemoryRegion(sdf, 'assistant_to_artist', 0x4_000)
    artist_to_assistant = MemoryRegion(sdf, 'artist_to_assistant', 0x4_000)

    mrs = [
        serial_mmio,
        assistant_to_artist,
        artist_to_assistant,
    ]

    for mr in mrs:
        sdf.add_mr(mr)

    serial_driver = ProtectionDomain(
        'serial_driver',
        'banscii-serial-driver.elf',
        priority=254,
        stack_size=DEFAULT_STACK_SIZE,
    )
    serial_driver.add_map(
        Map(
            serial_mmio,
            serial_driver.get_map_vaddr(serial_mmio),
            'rw',
            setvar_vaddr='serial_register_block',
        )
    )
    serial_driver.add_irq(
        Irq(
            board.serial_irq,
            setvar_id='serial_irq_id',
        )
    )

    assistant = ProtectionDomain(
        'assistant',
        'banscii-assistant.elf',
        priority=252,
        stack_size=DEFAULT_STACK_SIZE,
    )
    assistant.add_map(
        Map(
            artist_to_assistant,
            assistant.get_map_vaddr(artist_to_assistant),
            'r',
            setvar_vaddr='region_in_start',
            setvar_size='region_in_size',
        )
    )
    assistant.add_map(
        Map(
            assistant_to_artist,
            assistant.get_map_vaddr(assistant_to_artist),
            'rw',
            setvar_vaddr='region_out_start',
            setvar_size='region_out_size',
        )
    )

    artist = ProtectionDomain(
        'artist',
        'banscii-artist.elf',
        priority=253,
        stack_size=DEFAULT_STACK_SIZE,
    )
    artist.add_map(
        Map(
            assistant_to_artist,
            artist.get_map_vaddr(assistant_to_artist),
            'r',
            setvar_vaddr='region_in_start',
            setvar_size='region_in_size',
        )
    )
    artist.add_map(
        Map(
            artist_to_assistant,
            artist.get_map_vaddr(artist_to_assistant),
            'rw',
            setvar_vaddr='region_out_start',
            setvar_size='region_out_size',
        )
    )

    pds = [
        serial_driver,
        assistant,
        artist,
    ]

    for pd in pds:
        sdf.add_pd(pd)

    sdf.add_channel(
        Channel(
            serial_driver,
            assistant,
            pp_b=True,
            pd_a_setvar_id='assistant_channel_id',
            pd_b_setvar_id='serial_driver_channel_id',
        )
    )

    sdf.add_channel(
        Channel(
            assistant,
            artist,
            pp_a=True,
            pd_a_setvar_id='artist_channel_id',
            pd_b_setvar_id='assistant_channel_id',
        )
    )

    args.o.write(sdf.render())


if __name__ == '__main__':
    main()

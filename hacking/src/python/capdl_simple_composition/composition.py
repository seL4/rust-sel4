#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

import argparse
import json
import os
import shutil
import subprocess
import yaml
from collections import namedtuple
from pathlib import Path

from capdl import ObjectType, ObjectAllocator, CSpaceAllocator, AddressSpaceAllocator, Untyped, lookup_architecture, register_object_sizes
from capdl.Allocator import RenderState, Cap, ASIDTableAllocator, BestFitAllocator, RenderState

from capdl_simple_composition.kernel_config import KernelConfig
from capdl_simple_composition.device_tree import DeviceTree
from capdl_simple_composition.utils import aligned_chunks


class BaseComposition:

    @classmethod
    def from_env(cls):
        parser = argparse.ArgumentParser()
        parser.add_argument('--kernel', required=True, type=Path)
        parser.add_argument('--object-sizes', required=True, type=Path)
        parser.add_argument('--search-dir', required=True, type=Path, action='append')
        parser.add_argument('-o', required=True, type=Path)
        args = parser.parse_args()

        return cls(
            out_dir=args.o,
            kernel=args.kernel,
            object_sizes=args.object_sizes,
            search_dirs=args.search_dir,
        )

    def __init__(
        self,
        out_dir,
        kernel,
        object_sizes,
        search_dirs,
        compute_ut_covers=False,
    ):
        self.out_dir = out_dir

        self.kernel_config = KernelConfig(f'{kernel}/libsel4/include/kernel/gen_config.json')

        self.search_dirs = search_dirs

        self.device_tree = None
        self.platform_info = None
        if self.kernel_config.sel4_arch() != 'x86_64':
            device_tree_path = f'{kernel}/support/kernel.dtb'
            self.device_tree = DeviceTree(device_tree_path)
            platform_info_path = f'{kernel}/support/platform_gen.yaml'
            with open(platform_info_path) as f:
                self.platform_info = yaml.load(f, Loader=yaml.FullLoader)

        with open(object_sizes) as f:
            object_sizes = yaml.load(f, Loader=yaml.FullLoader)
            register_object_sizes(object_sizes)

        self.arch = self.kernel_config.sel4_arch()
        self.capdl_arch = self.kernel_config.capdl_arch()

        obj_space = ObjectAllocator()
        obj_space.spec.arch = self.arch
        self.render_state = RenderState(obj_space=obj_space)

        # TODO
        self.compute_ut_covers = compute_ut_covers

        self.components = set()
        self.files = {}

    def run(self):
        self.out_dir.mkdir(parents=True, exist_ok=True)
        self.compose()
        self.complete()

    def compose(self):
        raise NotImplementedError

    def register_component(self, component):
        self.components.add(component)

    def component(self, mk, name, *args, **kwargs):
        component = mk(self, name, *args, **kwargs)
        self.register_component(component)
        return component

    def alloc(self, *args, **kwargs):
        return self.obj_space().alloc(*args, **kwargs)

    def obj_space(self):
        return self.render_state.obj_space

    def register_file(self, fname, path):
        self.files[fname] = Path(path)
        return fname

    def get_file(self, fname):
        return self.files[fname]

    def spec(self):
        return self.render_state.obj_space.spec

    def finalize(self):
        for component in self.components:
            component.pre_finalize()
        for component in self.components:
            component.finalize()

    def allocate(self):
        ASIDTableAllocator().allocate(self.render_state.obj_space.spec)
        if self.compute_ut_covers:
            self.allocate_untyped()

    # TODO
    def allocate_untyped(self):
        ut_allocator = BestFitAllocator()

        def add_untyped_from(region, device=False):
            for paddr, size_bits in aligned_chunks(self.kernel_config.word_size(), region['start'], region['end']):
                ut_allocator.add_untyped(Untyped(name='root_untyped_{}'.format(
                    paddr), size_bits=size_bits, paddr=paddr))

        for region in self.platform_info['memory']:
            add_untyped_from(region)
        for region in self.platform_info['devices']:
            add_untyped_from(region, device=True)

        ut_allocator.allocate(self.render_state.obj_space.spec)

    def write_spec(self):
        (self.out_dir / 'spec.cdl').write_text(repr(self.spec()))

    def write_links(self):
        d = self.out_dir / 'links'
        shutil.rmtree(d, ignore_errors=True)  # HACK
        d.mkdir()
        for fname, path in self.files.items():
            (d / fname).symlink_to(path)

    def find_in_search_dirs(self, fname):
        for d in self.search_dirs:
            path = d / fname
            if path.exists():
                return path
        raise Exception('could not find {fname} in search dirs')

    def complete(self):
        self.finalize()
        self.allocate()
        self.write_spec()
        self.write_links()

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
        with open(os.environ['CONFIG']) as f:
            config = json.load(f)
        return cls(out_dir=os.environ['OUT_DIR'], config=config)

    def __init__(self, out_dir, config):
        self.out_dir = Path(out_dir)
        self.config = config

        self.kernel_config = KernelConfig(config['kernel_config'])
        self.device_tree = DeviceTree(config['device_tree'])

        platform_info = config.get('platform_info', None)
        if platform_info is not None:
            with open(platform_info) as f:
                self.platform_info = yaml.load(f, Loader=yaml.FullLoader)
        else:
            self.platform_info = None

        with open(config['object_sizes']) as f:
            object_sizes = yaml.load(f, Loader=yaml.FullLoader)
        register_object_sizes(object_sizes)

        self.arch = self.kernel_config.capdl_arch()

        obj_space = ObjectAllocator()
        obj_space.spec.arch = self.arch.capdl_name()
        self.render_state = RenderState(obj_space=obj_space)

        # TODO
        self.compute_ut_covers = config.get('compute_ut_covers', False)

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
                ut_allocator.add_untyped(Untyped(name='root_untyped_{}'.format(paddr), size_bits=size_bits, paddr=paddr))

        for region in self.platform_info['memory']:
            add_untyped_from(region)
        for region in self.platform_info['devices']:
            add_untyped_from(region, device=True)

        ut_allocator.allocate(self.render_state.obj_space.spec)

    def write_spec(self):
        (self.out_dir / 'spec.cdl').write_text(repr(self.spec()))

    def write_links(self):
        d = self.out_dir / 'links'
        shutil.rmtree(d, ignore_errors=True) # HACK
        d.mkdir()
        for fname, path in self.files.items():
            (d / fname).symlink_to(path)

    def complete(self):
        self.finalize()
        self.allocate()
        self.write_spec()
        self.write_links()

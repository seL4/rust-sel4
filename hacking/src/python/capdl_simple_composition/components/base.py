#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

from capdl import ObjectType, CSpaceAllocator, AddressSpaceAllocator, Cap

from capdl_simple_composition.utils import align_up, mk_fill

class BaseComponent:

    def __init__(self, composition, name):
        self.composition = composition
        self.key = name
        self.name = name

        cspace = self.alloc(ObjectType.seL4_CapTableObject, 'cspace')
        vspace = self.alloc(self.composition.capdl_arch.vspace().object, 'vspace')
        self.render_state().cspaces[self.key] = CSpaceAllocator(cspace)
        self.render_state().addr_spaces[self.key] = AddressSpaceAllocator('{}_addr_space'.format(self.key), vspace)
        self.render_state().pds[self.key] = vspace

    def render_state(self):
        return self.composition.render_state

    def obj_space(self):
        return self.render_state().obj_space

    def pd(self):
        return self.render_state().pds[self.key]

    def addr_space(self):
        return self.render_state().addr_spaces[self.key]

    def cspace(self):
        return self.render_state().cspaces[self.key]

    def cnode(self):
        return self.cspace().cnode

    def cnode_cap(self, update_guard_size=True):
        cap = Cap(self.cnode())
        if update_guard_size:
            self.cnode().update_guard_size_caps.append(cap)
        return cap

    def alloc(self, type, name, *args, **kwargs):
        name = '{}_{}'.format(self.name, name)
        return self.obj_space().alloc(type, name, *args, label=self.key, **kwargs)

    def pre_finalize(self):
        pass

    def finalize(self):
        self.cspace().cnode.finalise_size(arch=self.composition.capdl_arch)

    def fmt(self, s, *args):
        return s.format(self.name, *args)

    def config(self):
        return self.composition.config['components'][self.name]

    ###

    def map_with_size(self, size, vaddr, paddr=None, device=False, fill=[], label=None, read=False, write=False, execute=False, cached=True):
        assert vaddr % size == 0
        name = ''
        if label is not None:
            name += label + '_'
        name += '0x{:x}'.format(vaddr)
        frame = self.alloc(ObjectType.seL4_FrameObject, name, size=size, fill=fill, paddr=paddr, device=device)
        cap = Cap(frame, read=read, write=write, grant=execute, cached=cached)
        self.addr_space().add_hack_page(vaddr, size, cap)

    def map_page(self, *args, **kwargs):
        self.map_with_size(self.composition.kernel_config.page_size(), *args, **kwargs)

    def map_larger_page(self, *args, **kwargs):
        self.map_with_size(self.composition.kernel_config.larger_page_size(), *args, **kwargs)

    def map_range(self, vaddr_start, vaddr_end, label=None):
        vaddr = vaddr_start
        while vaddr < vaddr_end:
            size = None
            for page_size in reversed(self.composition.kernel_config.page_sizes()):
                if vaddr % page_size == 0 and vaddr + page_size <= vaddr_end:
                    size = page_size
                    break
            assert size is not None
            self.map_with_size(size, vaddr, label=label, read=True, write=True)
            vaddr += size

    def map_file(self, vaddr_start, fname: str, path, label=None):
        self.composition.register_file(fname, path)
        file_size = path.stat().st_size
        vaddr_end = vaddr_start + align_up(file_size, self.composition.kernel_config.page_size())
        vaddr = vaddr_start
        while vaddr < vaddr_end:
            size = None
            for page_size in reversed(self.composition.kernel_config.page_sizes()):
                if vaddr % page_size == 0 and vaddr + page_size <= vaddr_end:
                    size = page_size
                    break
            assert size is not None
            file_offset = vaddr - vaddr_start
            fill = mk_fill(0, min(size, file_size - file_offset), fname, file_offset)
            self.map_with_size(size, vaddr, label=label, read=True, write=True, fill=fill)
            vaddr += size
        return vaddr_end

    ###

    def get_cursor(self):
        return self.cur_vaddr

    def set_cursor(self, vaddr):
        self.cur_vaddr = vaddr

    def align(self, size):
        self.set_cursor(((self.get_cursor() - 1) | (size - 1)) + 1)
        return self.get_cursor()

    def align_to_page(self):
        return self.align(self.composition.kernel_config.page_size())

    def align_to_larger_page(self):
        return self.align(self.composition.kernel_config.larger_page_size())

    def advance(self, n, align=None):
        if align is not None:
            self.align(align)
        start = self.get_cursor()
        self.set_cursor(self.get_cursor() + n)
        end = self.get_cursor()
        return start, end

    def align_to_page_and_advance(self, n):
        return self.advance(n, align=self.composition.kernel_config.page_size())

    def align_to_larger_page_and_advance(self, n):
        return self.advance(n, align=self.composition.kernel_config.larger_page_size())

    def pad_and_align_to_page(self):
        n = self.composition.kernel_config.page_size()
        return self.advance(n, align=n)

    def pad_and_align_to_larger_page(self):
        n = self.composition.kernel_config.larger_page_size()
        return self.advance(n, align=n)

    def map_range_at_cursor(self, size, label=None):
        start = self.get_cursor()
        end = start + size
        self.map_range(start, end, label=label)
        self.set_cursor(end)
        return start, end

    def map_file_at_cursor(self, fname, path, label=None):
        start = self.get_cursor()
        end = self.map_file(start, fname, path, label=label)
        return start, end

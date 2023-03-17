import json

from capdl import lookup_architecture, register_object_sizes

class KernelConfig:

    def __init__(self, path):
        with open(path) as f:
            self.dict = json.load(f)

    def __getitem__(self, key):
        return self.dict[key]

    def arch(self):
        return self['ARCH']

    def sel4_arch(self):
        return self['SEL4_ARCH']

    def capdl_arch(self):
        return lookup_architecture(self.sel4_arch())

    def plat(self):
        return self['PLAT']

    def word_size(self):
        return int(self['WORD_SIZE'])

    def page_sizes_in_bits(self):
        if self.arch() == 'arm':
            return [12, 21]
        else:
            raise NotImplementedError

    def page_size_in_bits(self):
        return self.page_sizes_in_bits()[0]

    def larger_page_size_in_bits(self):
        assert self.arch() == 'arm'
        return self.page_sizes_in_bits()[1]

    def page_sizes(self):
        return [ 1 << n for n in self.page_sizes_in_bits() ]

    def page_size(self):
        return 1 << self.page_size_in_bits()

    def larger_page_size(self):
        return 1 << self.larger_page_size_in_bits()

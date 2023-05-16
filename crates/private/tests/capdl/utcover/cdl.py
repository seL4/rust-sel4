from capdl import ObjectType, Cap
from capdl_simple_composition import BaseComposition, ElfComponent

class TestComponent(ElfComponent):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.primary_thread.tcb['sc_slot'] = self.new_sched_context("primary")

        # ut = self.alloc(ObjectType.seL4_UntypedObject, size_bits=21, name='test_ut')
        # ut = self.alloc(ObjectType.seL4_UntypedObject, size_bits=21, paddr=0x62000000, name='test_ut')
        # frame1 = self.alloc(ObjectType.seL4_FrameObject, size=4096, name='test_frame1')
        frame2 = self.alloc(ObjectType.seL4_FrameObject, size=4096, name='test_frame2')
        # ut.add_child(frame1)
        # ut.add_child(frame2)

        self._arg = {
            'frame': self.cspace().alloc(frame2, read=True, write=True),
        }

    def arg_json(self):
        return self._arg

class TestComposition(BaseComposition):

    def compose(self):
        self.component(TestComponent, 'example_component')

TestComposition.from_env().run()

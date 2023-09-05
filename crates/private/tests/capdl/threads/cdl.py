from capdl import ObjectType, Cap
from capdl_simple_composition import BaseComposition, ElfComponent

class TestComponent(ElfComponent):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        max_affinity = int(self.composition.kernel_config['MAX_NUM_NODES']) - 1

        self.primary_thread.tcb['sc_slot'] = self.new_sched_context("primary")

        secondary_thread = self.secondary_thread('secondary_thread', affinity=max_affinity, sched_context=self.new_sched_context("secondary"))
        lock_nfn = self.alloc(ObjectType.seL4_NotificationObject, name='lock_nfn')
        barrier_nfn = self.alloc(ObjectType.seL4_NotificationObject, name='barrier_nfn')

        self._arg = {
            'lock_nfn': self.cspace().alloc(lock_nfn, read=True, write=True),
            'barrier_nfn': self.cspace().alloc(barrier_nfn, read=True, write=True),
            'secondary_thread': secondary_thread.endpoint,
            'foo': [1, 2, 3],
        }

    def arg_json(self):
        return self._arg

class TestComposition(BaseComposition):

    def compose(self):
        self.component(TestComponent, 'example_component')

TestComposition.from_env().run()

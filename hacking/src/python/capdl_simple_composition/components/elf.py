import json
import subprocess
from pathlib import Path
from elftools.elf.elffile import ELFFile

from capdl import ObjectType, ELF, Cap

from capdl_simple_composition.utils import mk_fill
from capdl_simple_composition.components.base import BaseComponent

DEFAULT_AFFINITY = 0
DEFAULT_PRIO = 128
DEFAULT_MAX_PRIO = 0
DEFAULT_STACK_SIZE = 2 * 2**20
DEFAULT_STATIC_HEAP_SIZE = 8 * 2**20

class ElfComponent(BaseComponent):

    def __init__(
            self, composition, name,
            update_guard_size=True,
            prio=DEFAULT_PRIO, max_prio=DEFAULT_MAX_PRIO,
            affinity=DEFAULT_AFFINITY,
            sched_context=None,
            **kwargs,
            ):

        super().__init__(composition, name, **kwargs)

        self.update_guard_size = update_guard_size

        elf_fname = '{}.elf'.format(self.name)
        elf_path = Path(self.config()['image'])
        self.elf_path = elf_path
        self.elf = ELF(str(elf_path), elf_fname, self.composition.arch)

        self.composition.register_file(elf_fname, elf_path)

        self.set_cursor(self.first_vaddr_after_elf())

        self.primary_thread = self.thread(
            'primary',
            affinity=affinity,
            sched_context=sched_context,
            prio=prio, max_prio=max_prio,
            alloc_endpoint=False,
            )

        self.secondary_threads = []

    def first_vaddr_after_elf(self):
        vaddr = 0
        for seg in self.elf._elf.iter_segments():
            if not seg['p_type'] == 'PT_LOAD':
                continue
            vaddr = max(vaddr, seg['p_vaddr'] + seg['p_memsz'])
        return vaddr

    def pre_finalize(self):
        self.runtime_config()
        super().pre_finalize()

    def finalize(self):
        elf_spec = self.elf.get_spec(infer_tcb=False, infer_asid=False, pd=self.addr_space().vspace_root, addr_space=self.addr_space())
        self.obj_space().merge(elf_spec, label=self.key)
        super().finalize()

    ###

    def threads(self):
        yield self.primary_thread
        yield from self.secondary_threads

    def num_threads(self):
        return 1 + len(self.secondary_threads)

    def thread(self, *args, **kwargs):
        thread = ElfThread(self, *args, **kwargs)
        self.register_thread(thread)
        return thread

    def secondary_thread(self, name, *args, affinity=None, prio=None, max_prio=None, **kwargs):
        if affinity is None:
            affinity = self.primary_thread.tcb.affinity
        if prio is None:
            prio = self.primary_thread.tcb.prio + 1 # a reasonable default for thin threads
        if max_prio is None:
            max_prio = self.primary_thread.tcb.max_prio
        thread = self.thread(name, *args, affinity=affinity, prio=prio, **kwargs)
        self.secondary_threads.append(thread)
        return thread

    def register_thread(self, thread):
        pass

    def new_sched_context(self, name, **kwargs):
        if not self.composition.kernel_config.is_mcs():
            return None
        return Cap(self.alloc(ObjectType.seL4_SchedContextObject, name, **kwargs))

    def static_heap_size(self):
        return self.config().get('heap_size', DEFAULT_STATIC_HEAP_SIZE)

    def static_heap(self):
        size = self.static_heap_size()
        self.pad_and_align_to_larger_page()
        start, end = self.map_range_at_cursor(size, label='heap')
        lock = self.cspace().alloc(
                self.alloc(ObjectType.seL4_NotificationObject, 'heap_lock'),
                read=True, write=True, badge=1,
                )
        return {
            'start': start,
            'end': end,
            'lock': lock,
            }

    def runtime_config(self):
        heap_info = self.static_heap()
        arg_bin = self.arg()

        config = {
            'static_heap': {
                'start': heap_info['start'],
                'end': heap_info['end'],
                },
            'static_heap_mutex_notification': heap_info['lock'],
            'idle_notification': self.cspace().alloc(
                self.alloc(ObjectType.seL4_NotificationObject, 'idle_notification'),
                read=True,
                ),
            'threads': [ thread.get_thread_runtime_config() for thread in self.threads() ],
            'image_identifier': str(self.elf_path),
            'arg': str(self.composition.out_dir / arg_bin),
        }

        path_base = self.composition.out_dir / '{}_config'.format(self.name)
        path_bin = path_base.with_suffix('.bin')
        path_json = path_base.with_suffix('.json')
        with path_json.open('w') as f_json:
            json.dump(config, f_json, indent=4)
        with path_json.open('r') as f_json:
            with path_bin.open('wb') as f_bin:
                subprocess.check_call(['sel4-simple-task-serialize-runtime-config'], stdin=f_json, stdout=f_bin)

        self.composition.register_file(path_bin.name, path_bin)

        config_size = path_bin.stat().st_size

        self.pad_and_align_to_page()
        config_vaddr, _ = self.map_file_at_cursor(path_bin.name, path_bin, 'config')

        for i, thread in enumerate(self.threads()):
            thread.set_component_runtime_config(config_vaddr, config_size, i)

    def arg(self):
        path_base = self.composition.out_dir / '{}_arg'.format(self.name)
        path_bin = path_base.with_suffix('.bin')
        path_json = path_base.with_suffix('.json')
        with path_json.open('w') as f_json:
            json.dump(self.arg_json(), f_json, indent=4)
        with path_json.open('r') as f_json:
            with path_bin.open('wb') as f_bin:
                try:
                    subprocess.check_call(self.transform_arg_json_argv(), stdin=f_json, stdout=f_bin)
                except Exception as e:
                    print(path_json)
                    raise e
        self.composition.register_file(path_bin.name, path_bin)
        return path_bin.name

    def empty_arg(self):
        path_base = self.composition.out_dir / '{}_arg'.format(self.name)
        path_bin = path_base.with_suffix('.bin')
        with path_bin.open('wb') as f_bin:
            pass
        self.composition.register_file(path_bin.name, path_bin)
        return path_bin.name

    def arg_json(self):
        return {}

    def transform_arg_json_argv(self):
        return ['cat']


class ElfThread:

    def __init__(self, component, name,
            stack_size=DEFAULT_STACK_SIZE,
            prio=DEFAULT_PRIO, max_prio=DEFAULT_MAX_PRIO,
            affinity=DEFAULT_AFFINITY,
            sched_context=None,
            alloc_endpoint=True,
            ):

        self.component = component
        self.name = name

        page_size = self.component.composition.kernel_config.page_size()
        self.component.pad_and_align_to_larger_page()
        stack_start_vaddr, stack_end_vaddr = self.component.map_range_at_cursor(stack_size, label='{}_stack'.format(self.name))
        self.component.pad_and_align_to_larger_page()
        ipc_buffer_vaddr, _ = self.component.advance(page_size)
        ipc_buffer_frame = self.component.alloc(ObjectType.seL4_FrameObject, '{}_ipc_buffer'.format(self.name), size=page_size)
        ipc_buffer_cap = Cap(ipc_buffer_frame, read=True, write=True)
        self.component.addr_space().add_hack_page(ipc_buffer_vaddr, page_size, ipc_buffer_cap)

        tcb = self.component.alloc(ObjectType.seL4_TCBObject, name='{}_tcb'.format(self.name))
        tcb.ip = self.component.elf.get_entry_point()
        tcb.sp = stack_end_vaddr
        tcb.addr = ipc_buffer_vaddr
        tcb.prio = prio
        tcb.max_prio = max_prio
        tcb.affinity = affinity
        tcb.resume = True
        tcb['cspace'] = self.component.cnode_cap(update_guard_size=self.component.update_guard_size)
        tcb['vspace'] = Cap(self.component.pd())
        tcb['ipc_buffer_slot'] = ipc_buffer_cap
        tcb['sc_slot'] = sched_context

        if alloc_endpoint:
            endpoint = self.component.cspace().alloc(
                self.component.alloc(ObjectType.seL4_EndpointObject, name='{}_thread_ep'.format(self.name)),
                read=True, write=True,
                )
        else:
            endpoint = None

        self.tcb = tcb
        self.ipc_buffer_vaddr = ipc_buffer_vaddr
        self.endpoint = endpoint

    def full_name(self):
        return '{}_thread_{}'.format(self.component.name, self.name)

    def get_thread_runtime_config(self):
        if self.component.composition.kernel_config.is_mcs():
            reply_authority = self.component.cspace().alloc(
                self.component.alloc(ObjectType.seL4_RTReplyObject, name='{}_reply'.format(self.name)),
                )
        else:
            reply_authority = None

        return {
            'ipc_buffer_addr': self.ipc_buffer_vaddr,
            'endpoint': self.endpoint,
            'reply_authority': reply_authority,
            }

    def set_component_runtime_config(self, config_vaddr, config_size, thread_index):
        self.tcb.init.append(config_vaddr)
        self.tcb.init.append(config_size)
        self.tcb.init.append(thread_index)

from capdl import ObjectType, Cap, ARMIRQMode
from capdl_simple_composition import BaseComposition, ElfComponent

# EVENT_NFN_BADGE_IRQ = 1 << 0

class TestComponent(ElfComponent):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.primary_thread.tcb['sc_slot'] = self.new_sched_context("primary")

        event_nfn = self.alloc(ObjectType.seL4_NotificationObject, name='event_nfn')

        irq_handlers = []
        spi_base = 32
        irq_range_start, irq_range_end = spi_base + 0x10, spi_base + 0x2f + 1
        irq_trigger = ARMIRQMode.seL4_ARM_IRQ_EDGE
        for i in range(irq_range_start, irq_range_end):
            irq_handlers.append(self.alloc(
                ObjectType.seL4_IRQHandler,
                name='irq_{}_handler'.format(i),
                number=i, trigger=irq_trigger,
                notification=Cap(event_nfn, badge=1 << (i - irq_range_start)),
                ))

        mmio_range_size = 0x4000
        mmio_paddr_range_start = 0xa000000
        # self.pad_and_align_to_page()
        # mmio_vaddr_range_start, mmio_vaddr_range_end = self.advance(mmio_range_size)
        mmio_vaddr_range_start = mmio_paddr_range_start
        mmio_vaddr_range_end = mmio_vaddr_range_start + mmio_range_size
        for i in range(4):
            self.map_page(
                mmio_vaddr_range_start + i * self.composition.kernel_config.page_size(),
                paddr=mmio_paddr_range_start + i * self.composition.kernel_config.page_size(),
                device=True,
                read=True, write=True, cached=False,
                label='mmio_frame',
                )

        dma_paddr_range_start = 0x62000000
        # self.pad_and_align_to_larger_page()
        # dma_vaddr_range_start, dma_vaddr_range_end = self.advance(self.composition.kernel_config.larger_page_size())
        dma_vaddr_range_start = dma_paddr_range_start
        dma_vaddr_range_end = dma_vaddr_range_start + self.composition.kernel_config.larger_page_size()
        self.map_larger_page(
            dma_vaddr_range_start,
            paddr=dma_paddr_range_start,
            read=True, write=True, cached=True,
            label='dma_frame',
            )

        timer_mmio_paddr = 0x90d0000
        self.pad_and_align_to_page()
        timer_mmio_vaddr, _ = self.advance(4096)
        self.map_page(
            timer_mmio_vaddr,
            paddr=timer_mmio_paddr,
            device=True,
            read=True, write=True, cached=False,
            label='timer_mmio_frame',
            )

        TIMER_IRQ_BADGE = 60

        timer_irq = spi_base + 0x0b
        timer_irq_handler = self.alloc(
            ObjectType.seL4_IRQHandler,
            name='timer_irq_{}_handler'.format(timer_irq),
            number=timer_irq, trigger=ARMIRQMode.seL4_ARM_IRQ_LEVEL,
            notification=Cap(event_nfn, badge=1 << TIMER_IRQ_BADGE),
            )

        self._arg = {
            'event_nfn': self.cspace().alloc(event_nfn, read=True),
            'virtio_irq_range': {
                'start': irq_range_start,
                'end': irq_range_end,
                },
            'virtio_irq_handlers': [ self.cspace().alloc(irq_handler) for irq_handler in irq_handlers ],
            'virtio_mmio_vaddr_range': {
                'start': mmio_vaddr_range_start,
                'end': mmio_vaddr_range_end,
                },
            'virtio_dma_vaddr_range': {
                'start': dma_vaddr_range_start,
                'end': dma_vaddr_range_end,
                },
            'virtio_dma_vaddr_to_paddr_offset': dma_paddr_range_start - dma_vaddr_range_start,
            'timer_mmio_vaddr': timer_mmio_vaddr,
            # 'timer_freq': 24 * 10**6,
            'timer_freq': 10**6, # HACK
            'timer_irq_handler': self.cspace().alloc(timer_irq_handler),
            }

    def arg_json(self):
        return self._arg

class TestComposition(BaseComposition):

    def compose(self):
        self.component(TestComponent, 'example_component')

TestComposition.from_env().run()

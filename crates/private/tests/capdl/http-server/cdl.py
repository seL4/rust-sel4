from capdl import ObjectType, Cap, ARMIRQMode
from capdl_simple_composition import BaseComposition, ElfComponent

TIMER_IRQ_BADGE = 1 << 0
VIRTIO_NET_IRQ_BADGE = 1 << 1
VIRTIO_BLK_IRQ_BADGE = 1 << 2

class TestComponent(ElfComponent):

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.primary_thread.tcb['sc_slot'] = self.new_sched_context("primary")

        event_nfn = self.alloc(ObjectType.seL4_NotificationObject, name='event_nfn')

        spi_base = 32

        timer_irq = spi_base + 0x0b
        timer_irq_handler = self.alloc(
            ObjectType.seL4_IRQHandler,
            name='timer_irq_{}_handler'.format(timer_irq),
            number=timer_irq, trigger=ARMIRQMode.seL4_ARM_IRQ_LEVEL,
            notification=Cap(event_nfn, badge=TIMER_IRQ_BADGE),
            )

        timer_mmio_paddr = 0x90d0000
        self.pad_and_align_to_page()
        timer_mmio_vaddr, _ = self.advance(self.composition.kernel_config.page_size())
        self.map_page(
            timer_mmio_vaddr,
            paddr=timer_mmio_paddr,
            device=True,
            read=True, write=True, cached=False,
            label='timer_mmio_frame',
            )

        # timer_freq = 24 * 10**6
        timer_freq = 10**6 # HACK

        virtio_net_irq = spi_base + 0x2f
        virtio_net_irq_handler = self.alloc(
            ObjectType.seL4_IRQHandler,
            name='virtio_net_irq_{}_handler'.format(virtio_net_irq),
            number=virtio_net_irq, trigger=ARMIRQMode.seL4_ARM_IRQ_EDGE,
            notification=Cap(event_nfn, badge=VIRTIO_NET_IRQ_BADGE),
            )

        virtio_blk_irq = spi_base + 0x2e
        virtio_blk_irq_handler = self.alloc(
            ObjectType.seL4_IRQHandler,
            name='virtio_blk_irq_{}_handler'.format(virtio_blk_irq),
            number=virtio_blk_irq, trigger=ARMIRQMode.seL4_ARM_IRQ_EDGE,
            notification=Cap(event_nfn, badge=VIRTIO_BLK_IRQ_BADGE),
            )

        virtio_mmio_paddr = 0xa003000
        self.pad_and_align_to_page()
        virtio_mmio_vaddr, _ = self.advance(self.composition.kernel_config.page_size())
        self.map_page(
            virtio_mmio_vaddr,
            paddr=virtio_mmio_paddr,
            device=True,
            read=True, write=True, cached=False,
            label='virtio_mmio_frame',
            )
        virtio_net_mmio_vaddr = virtio_mmio_vaddr
        virtio_blk_mmio_vaddr = virtio_mmio_vaddr
        virtio_net_mmio_offset = 0xe00
        virtio_blk_mmio_offset = 0xc00

        _, virtio_dma_vaddr_range_start = self.pad_and_align_to_larger_page()
        virtio_dma_paddr_range_start = 0x62000000
        for i in range(64):
            paddr = virtio_dma_paddr_range_start + i * 2**21
            vaddr, _ = self.advance(self.composition.kernel_config.larger_page_size())
            self.map_larger_page(
                vaddr,
                paddr=paddr,
                read=True, write=True, cached=True,
                label='virtio_dma_region',
                )
        virtio_dma_vaddr_range_end = self.get_cursor()

        self._arg = {
            'event_nfn': self.cspace().alloc(event_nfn, read=True),
            'timer_irq_handler': self.cspace().alloc(timer_irq_handler),
            'timer_mmio_vaddr': timer_mmio_vaddr,
            'timer_freq': timer_freq,
            'virtio_net_irq_handler': self.cspace().alloc(virtio_net_irq_handler),
            'virtio_net_mmio_vaddr': virtio_net_mmio_vaddr,
            'virtio_net_mmio_offset': virtio_net_mmio_offset,
            'virtio_blk_irq_handler': self.cspace().alloc(virtio_blk_irq_handler),
            'virtio_blk_mmio_vaddr': virtio_blk_mmio_vaddr,
            'virtio_blk_mmio_offset': virtio_blk_mmio_offset,
            'virtio_dma_vaddr_range': {
                'start': virtio_dma_vaddr_range_start,
                'end': virtio_dma_vaddr_range_end,
                },
            'virtio_dma_vaddr_to_paddr_offset': virtio_dma_paddr_range_start - virtio_dma_vaddr_range_start,
            }

    def arg_json(self):
        return self._arg

class TestComposition(BaseComposition):

    def compose(self):
        self.component(TestComponent, 'example_component')

TestComposition.from_env().run()

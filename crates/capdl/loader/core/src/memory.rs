use core::ops::Range;

use sel4::{cap_type, BootInfo, FrameSize};

#[repr(align(4096))]
struct SmallPagePlaceHolder([u8; FrameSize::Small.bytes()]);

static SMALL_PAGE_PLACEHOLDER: SmallPagePlaceHolder =
    SmallPagePlaceHolder([0; FrameSize::Small.bytes()]);

pub(crate) fn init_copy_addrs(
    bootinfo: &BootInfo,
    user_image_bounds: &Range<usize>,
) -> Result<(usize, usize), sel4::Error> {
    let small_frame_copy_addr = {
        let addr = addr_of_ref(&SMALL_PAGE_PLACEHOLDER);
        assert_eq!(addr % FrameSize::Small.bytes(), 0);
        let num_user_frames =
            usize::try_from(bootinfo.user_image_frames().end - bootinfo.user_image_frames().start)
                .unwrap();
        let user_image_footprint = coarsen_footprint(user_image_bounds, FrameSize::Small.bytes());
        assert_eq!(
            user_image_footprint.len(),
            num_user_frames * FrameSize::Small.bytes()
        );
        let ix = (addr - user_image_footprint.start) / FrameSize::Small.bytes();
        let cap = BootInfo::init_cspace_local_cptr::<cap_type::SmallPage>(
            bootinfo.user_image_frames().start + ix,
        );
        cap.frame_unmap()?;
        addr
    };
    let large_frame_copy_addr = {
        let addr_space_footprint = coarsen_footprint(
            &(user_image_bounds.start..bootinfo.footprint().end),
            FrameSize::Large.bytes(),
        );
        match (
            addr_space_footprint.start % FrameSize::Huge.bytes(),
            addr_space_footprint.end % FrameSize::Huge.bytes(),
        ) {
            (0, 0) => panic!(), // absurd
            (_, 0) => {
                round_down(addr_space_footprint.start, FrameSize::Large.bytes())
                    - FrameSize::Large.bytes()
            }
            (_, _) => addr_space_footprint
                .end
                .next_multiple_of(FrameSize::Large.bytes()),
        }
    };
    Ok((small_frame_copy_addr, large_frame_copy_addr))
}

fn coarsen_footprint(footprint: &Range<usize>, granularity: usize) -> Range<usize> {
    round_down(footprint.start, granularity)..footprint.end.next_multiple_of(granularity)
}

const fn round_down(n: usize, b: usize) -> usize {
    n - n % b
}

fn addr_of_ref<T>(x: &T) -> usize {
    (x as *const T).addr()
}

use r0::zero_bss;
use libregister::{
    VolatileCell,
    RegisterR, RegisterW, RegisterRW,
};
use libcortex_a9::{asm, regs::*, cache, mmu};
use libboard_zynq::{slcr, mpcore};

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __stack_start: u32;
    fn main_core0();
    fn main_core1();
}

/// `0` means: wait for initialization by core0
static mut CORE1_STACK: VolatileCell<u32> = VolatileCell::new(0);

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _boot_cores() -> ! {
    const CORE_MASK: u32 = 0x3;

    match MPIDR.read() & CORE_MASK {
        0 => {
            SP.write(&mut __stack_start as *mut _ as u32);
            boot_core0();
        }
        1 => {
            while CORE1_STACK.get() == 0 {
                asm::wfe();
            }

            SP.write(CORE1_STACK.get());
            boot_core1();
        }
        _ => unreachable!(),
    }
}

#[naked]
#[inline(never)]
unsafe fn boot_core0() -> ! {
    l1_cache_init();

    let mpcore = mpcore::RegisterBlock::new();
    mpcore.scu_invalidate.invalidate_all_cores();

    zero_bss(&mut __bss_start, &mut __bss_end);

    let mmu_table = mmu::L1Table::get()
        .setup_flat_layout();
    mmu::with_mmu(mmu_table, || {
        mpcore.scu_control.start();
        ACTLR.enable_smp();
        // TODO: Barriers reqd when core1 is not yet starting?
        asm::dmb();
        asm::dsb();

        main_core0();
        panic!("return from main");
    });
}

#[naked]
#[inline(never)]
unsafe fn boot_core1() -> ! {
    l1_cache_init();

    let mpcore = mpcore::RegisterBlock::new();
    mpcore.scu_invalidate.invalidate_core1();

    let mmu_table = mmu::L1Table::get();
    mmu::with_mmu(mmu_table, || {
        ACTLR.enable_smp();
        // TODO: Barriers reqd when core1 is not yet starting?
        asm::dmb();
        asm::dsb();

        main_core1();
        panic!("return from main_core1");
    });
}

fn l1_cache_init() {
    use libcortex_a9::cache::*;

    // Invalidate TLBs
    tlbiall();
    // Invalidate I-Cache
    iciallu();
    // Invalidate Branch Predictor Array
    bpiall();
    // Invalidate D-Cache
    //
    // NOTE: It is both faster and correct to only invalidate instead
    //       of also flush the cache (as was done before with
    //       `dccisw()`) and it is correct to perform this operation
    //       for all of the L1 data cache rather than a (previously
    //       unspecified) combination of one cache set and one cache
    //       way.
    dciall();
}

pub struct Core1<S: AsMut<[u32]>> {
    pub stack: S,
}

impl<S: AsMut<[u32]>> Core1<S> {
    pub fn reset(&self) {
        unsafe {
            CORE1_STACK.set(0);
        }

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
        });
    }

    /// Reset and start core1
    ///
    /// The stack must not be in OCM because core1 still has to
    /// initialize its MMU before it can access DDR.
    pub fn start(stack: S) -> Self {
        let mut core = Core1 { stack };
        
        // reset and stop (safe to repeat)
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
        });

        let stack = core.stack.as_mut();
        let stack_start = &mut stack[stack.len() - 1];
        unsafe {
            CORE1_STACK.set(stack_start as *mut _ as u32);
        }
        // Ensure stack pointer has been written to cache
        asm::dmb();
        // Flush cache-line
        cache::dccmvac(unsafe { &CORE1_STACK } as *const _ as u32);

        // wake up core1
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(false));
        });

        core
    }
}

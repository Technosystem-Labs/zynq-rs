use r0::zero_bss;
use core::ptr::write_volatile;
use libregister::{
    VolatileCell,
    RegisterR, RegisterW, RegisterRW,
};
use libcortex_a9::{asm, regs::*, cache, mmu};
use libboard_zynq::{slcr, mpcore};

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __stack0_start: u32;
    static mut __stack1_start: u32;
    fn main_core0();
    fn main_core1();
}

static mut CORE1_ENABLED: VolatileCell<bool> = VolatileCell::new(false);

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn Reset() -> ! {
    match MPIDR.read().cpu_id() {
        0 => {
            SP.write(&mut __stack0_start as *mut _ as u32);
            boot_core0();
        }
        1 => {
            while !CORE1_ENABLED.get() {
                asm::wfe();
            }
            SP.write(&mut __stack1_start as *mut _ as u32);
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

pub struct Core1 {
}

impl Core1 {
    /// Reset and start core1
    pub fn start(sdram: bool) -> Self {
        // reset and stop (safe to repeat)
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
        });

        if sdram {
            // Cores always start from OCM no matter what you do.
            // Make up a vector table there that just jumps to SDRAM.
            for i in 0..8 {
                unsafe {
                    // this is the ARM instruction "b +0x00100000"
                    write_volatile((i*4) as *mut u32, 0xea03fffe);
                }
            }
        }

        unsafe {
            CORE1_ENABLED.set(true);
        }
        // Ensure values have been written to cache
        asm::dmb();
        // Flush cache-line
        cache::dccmvac(unsafe { &CORE1_ENABLED } as *const _ as usize);
        if sdram {
            cache::dccmvac(0);
        }

        // wake up core1
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(false));
        });

        Core1 {}
    }

    pub fn disable(&self) {
        unsafe {
            CORE1_ENABLED.set(false);
            cache::dccmvac(&CORE1_ENABLED  as *const _ as usize);
            asm::dsb();
        }
        self.restart();
    }

    pub fn restart(&self) {
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(true));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(false));
        });
    }
}

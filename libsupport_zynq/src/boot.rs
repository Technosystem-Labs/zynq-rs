use core::{arch::naked_asm, ptr::write_volatile};

use libboard_zynq::{mpcore, slcr};
use libcortex_a9::{asm, cache, enable_fpu, interrupt_handler, l2c, mmu, notify_spin_lock, regs::*, spin_lock_yield};
use libregister::{RegisterR, RegisterRW, VolatileCell};
use r0::zero_bss;

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __stack0_start: u32;
    static mut __stack1_start: u32;
    fn main_core0();
    fn main_core1();
}

static mut CORE1_ENABLED: VolatileCell<bool> = VolatileCell::new(false);

interrupt_handler!(Reset, reset_irq, __stack0_start, __stack1_start, {
    // no need to setup stack here, as we already did when entering the handler
    match MPIDR.read().cpu_id() {
        0 => {
            boot_core0();
        }
        1 => {
            #[allow(static_mut_refs)]
            while !CORE1_ENABLED.get() {
                spin_lock_yield();
            }
            boot_core1();
        }
        _ => unreachable!(),
    }
});

#[allow(static_mut_refs)]
#[inline(never)]
unsafe extern "C" fn boot_core0() -> ! {
    l1_cache_init();

    enable_fpu();
    let mpcore = mpcore::RegisterBlock::mpcore();
    mpcore.scu_invalidate.invalidate_all_cores();

    zero_bss(&raw mut __bss_start, &raw mut __bss_end);

    let mmu_table = mmu::L1Table::get().setup_flat_layout();
    cache::dcc(mmu_table);
    mmu::with_mmu(mmu_table, || {
        mpcore.scu_control.start();
        ACTLR.enable_smp();
        ACTLR.enable_prefetch();
        // TODO: Barriers reqd when core1 is not yet starting?
        asm::dmb();
        asm::dsb();

        asm::enable_fiq();
        asm::enable_irq();
        main_core0();
        panic!("return from main");
    });
}

#[inline(never)]
unsafe extern "C" fn boot_core1() -> ! {
    l1_cache_init();

    enable_fpu();
    let mpcore = mpcore::RegisterBlock::mpcore();
    mpcore.scu_invalidate.invalidate_core1();

    let mmu_table = mmu::L1Table::get();
    mmu::with_mmu(mmu_table, || {
        ACTLR.enable_smp();
        ACTLR.enable_prefetch();
        // TODO: Barriers reqd when core1 is not yet starting?
        asm::dmb();
        asm::dsb();

        asm::enable_fiq();
        asm::enable_irq();
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
    dciall_l1();
}

pub struct Core1 {}

#[allow(static_mut_refs)]
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
                    write_volatile((i * 4) as *mut u32, 0xea03fffe);
                }
            }
        }

        unsafe {
            CORE1_ENABLED.set(true);
        }
        // Flush cache-line
        cache::dcc(&raw const CORE1_ENABLED);
        if sdram {
            cache::dccmvac(0);
            asm::dsb();
            l2c::l2_cache_clean(0);
            l2c::l2_cache_sync();
        }

        // wake up core1
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_rst1(false));
            slcr.a9_cpu_rst_ctrl.modify(|_, w| w.a9_clkstop1(false));
        });
        notify_spin_lock();

        Core1 {}
    }

    pub fn disable(&self) {
        unsafe {
            CORE1_ENABLED.set(false);
            cache::dccmvac((&raw const CORE1_ENABLED).addr());
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

use r0::zero_bss;
use crate::regs::{RegisterR, RegisterW};
use crate::cortex_a9::{asm, regs::*, mmu};
use crate::zynq::mpcore;

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __stack_start: u32;
}

static mut CORE1_STACK: u32 = 0;

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
            // Wait for a first `sev` so that `CORE1_STACK` is cleared
            // by `zero_bss()` on core 0.
            asm::wfe();

            while CORE1_STACK == 0 {
                asm::wfe();
            }

            SP.write(CORE1_STACK);
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

        crate::main();
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

        crate::main_core1();
        panic!("return from main_core1");
    });
}

fn l1_cache_init() {
    use crate::cortex_a9::cache::*;

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

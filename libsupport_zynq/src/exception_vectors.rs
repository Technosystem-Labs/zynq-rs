use libregister::{RegisterR, RegisterW};
use libcortex_a9::{regs::{DFSR, MPIDR, VBAR}, interrupt_handler};
use libboard_zynq::{println, stdio}; 
use core::arch::naked_asm;

pub fn set_vector_table(base_addr: u32){
    VBAR.write(base_addr);
}

interrupt_handler!(UndefinedInstruction, undefined_instruction, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("UndefinedInstruction");
    loop {}
});

interrupt_handler!(SoftwareInterrupt, software_interrupt, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("SoftwareInterrupt");
    loop {}
});

interrupt_handler!(PrefetchAbort, prefetch_abort, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("PrefetchAbort");
    loop {}
});

interrupt_handler!(DataAbort, data_abort, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();

    println!("DataAbort on core {}", MPIDR.read().cpu_id());
    println!("DFSR: {:03X}", DFSR.read());

    loop {}
});

interrupt_handler!(ReservedException, reserved_exception, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("ReservedException");
    loop {}
});

#[cfg(feature = "dummy_irq_handler")]
interrupt_handler!(IRQ, irq, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("IRQ");
    loop {}
});

#[cfg(feature = "dummy_fiq_handler")]
interrupt_handler!(FIQ, fiq, __irq_stack0_start, __irq_stack1_start, {
    stdio::drop_uart();
    println!("FIQ");
    loop {}
});

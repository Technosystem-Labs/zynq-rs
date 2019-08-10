use bit_field::BitField;
use super::{regs::*, asm};
use crate::regs::RegisterW;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum AccessDomain {
    NoAccess = 0b00,
    Client = 0b01,
    _Reserved = 0b10,
    Manager = 0b11,
}

const ACCESS_DOMAINS_SIZE: usize = 16;
pub struct AccessDomains([AccessDomain; ACCESS_DOMAINS_SIZE]);

impl AccessDomains {
    pub fn all_manager() -> Self {
        AccessDomains([AccessDomain::Manager; ACCESS_DOMAINS_SIZE])
    }
}

impl Into<u32> for AccessDomains {
    fn into(self) -> u32 {
        let mut result = 0;
        for (i, domain) in self.0.iter().enumerate() {
            result |= (*domain as u32) << (2 * i);
        }
        result
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum AccessPermissions {
    PermissionFault = 0,
    PrivilegedOnly,
    NoUserWrite,
    FullAccess,
    _Reserved1,
    PrivilegedReadOnly,
    ReadOnly,
    _Reserved2
}

impl AccessPermissions {
    fn ap(&self) -> u8 {
        (*self as u8) & 0b11
    }

    fn apx(&self) -> bool {
        (*self as u8) > (AccessPermissions::FullAccess as u8)
    }
}

pub struct L1Section {
    pub global: bool,
    pub shareable: bool,
    pub access: AccessPermissions,
    /// Type EXtension
    pub tex: u8,
    pub domain: u8,
    pub exec: bool,
    pub cacheable: bool,
    pub bufferable: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct L1Entry(u32);

impl L1Entry {
    #[inline(always)]
    pub fn section(phys_base: u32, section: L1Section) -> Self {
        // Must be aligned to 1 MB
        assert!(phys_base & 0x000f_ffff == 0);
        let mut entry = L1Entry(phys_base);

        entry.0.set_bits(0..=1, 0b10);
        entry.0.set_bit(2, section.bufferable);
        entry.0.set_bit(3, section.cacheable);
        entry.0.set_bit(4, !section.exec);
        assert!(section.domain < 16);
        entry.0.set_bits(5..=8, section.domain.into());
        entry.0.set_bits(10..=11, section.access.ap().into());
        assert!(section.tex < 8);
        entry.0.set_bits(12..=14, section.tex.into());
        entry.0.set_bit(15, section.access.apx());
        entry.0.set_bit(16, section.shareable);
        entry.0.set_bit(17, !section.global);

        entry
    }
}

const L1_TABLE_SIZE: usize = 4096;
#[doc(hidden)]
#[link_section = ".bss.l1_table"]
#[no_mangle]
pub static mut l1_table: L1Table = L1Table {
    table: [L1Entry(0); L1_TABLE_SIZE]
};

/// The `#[repr(align(16384))]` is unfortunately ineffective. Hence we
/// require explicit linking to a region defined in the linker script.
#[repr(align(16384))]
pub struct L1Table {
    table: [L1Entry; L1_TABLE_SIZE]
}

impl L1Table {
    pub fn get() -> &'static mut Self {
        unsafe {
            &mut l1_table
        }
    }

    pub fn setup_flat_layout(&mut self) -> &Self {
        /* 0x00000000 - 0x00100000 (cacheable) */
        self.direct_mapped_section(0, L1Section {
            global: true,
            shareable: true,
            access: AccessPermissions::FullAccess,
            tex: 0b101,
            domain: 0b1111,
            exec: true,
            cacheable: false,
            bufferable: true,
        });
        /* (DDR cacheable) */
        for ddr in 1..=0x1ff {
            self.direct_mapped_section(ddr, L1Section {
                global: true,
                shareable: true,
                access: AccessPermissions::FullAccess,
                tex: 0b101,
                domain: 0b1111,
                exec: true,
                cacheable: true,
                bufferable: true,
            });
        }
        /* (unassigned/reserved). */
        for undef in 0x1ff..=0x3ff {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0x40000000 - 0x7fffffff (FPGA slave0) */
        for fpga_slave in 0x400..=0x7ff {
            self.direct_mapped_section(fpga_slave, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0x80000000 - 0xbfffffff (FPGA slave1) */
        for fpga_slave in 0x800..=0xbff {
            self.direct_mapped_section(fpga_slave, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xc0000000 - 0xdfffffff (unassigned/reserved). */
        for undef in 0xc00..=0xdff {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xe0000000 - 0xe02fffff (Memory mapped devices)
         * UART/USB/IIC/SPI/CAN/GEM/GPIO/QSPI/SD/NAND */
        for mmapped_dev in 0xe00..=0xe02 {
            self.direct_mapped_section(mmapped_dev, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: false,
                bufferable: true,
            });
        }
        /* 0xe0300000 - 0xe0ffffff (unassigned/reserved). */
        for undef in 0xe03..=0xe0f {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xe1000000 - 0xe1ffffff (NAND) */
        for nand in 0xe10..=0xe1f {
            self.direct_mapped_section(nand, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: false,
                bufferable: true,
            });
        }
        /* 0xe2000000 - 0xe3ffffff (NOR) */
        for nor in 0xe20..=0xe3f {
            self.direct_mapped_section(nor, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: false,
                bufferable: true,
            });
        }
        /* 0xe4000000 - 0xe5ffffff (SRAM) */
        for nor in 0xe40..=0xe5f {
            self.direct_mapped_section(nor, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: true,
                bufferable: true,
            });
        }
        /* 0xe6000000 - 0xf7ffffff (unassigned/reserved). */
        for undef in 0xe60..=0xf7f {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xf8000000 - 0xf8ffffff (AMBA APB Peripherals) */
        for apb in 0xf80..=0xf8f {
            self.direct_mapped_section(apb, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: false,
                bufferable: true,
            });
        }
        /* 0xf9000000 - 0xfbffffff (unassigned/reserved). */
        for undef in 0xf90..=0xfbf {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xfc000000 - 0xfdffffff (Linear QSPI - XIP) */
        for qspi in 0xfc0..=0xfdf {
            self.direct_mapped_section(qspi, L1Section {
                global: true,
                shareable: false,
                access: AccessPermissions::FullAccess,
                tex: 0,
                domain: 0,
                exec: true,
                cacheable: false,
                bufferable: true,
            });
        }
        /* 0xfe000000 - 0xffefffff (unassigned/reserved). */
        for undef in 0xfe0..=0xffe {
            self.direct_mapped_section(undef, L1Section {
                global: false,
                shareable: false,
                access: AccessPermissions::PermissionFault,
                tex: 0,
                domain: 0,
                exec: false,
                cacheable: false,
                bufferable: false,
            });
        }
        /* 0xfff00000 - 0xffffffff (256K OCM when mapped to high address space) */
        self.direct_mapped_section(0xfff, L1Section {
            global: true,
            shareable: false,
            access: AccessPermissions::FullAccess,
            tex: 0b100,
            domain: 0,
            exec: true,
            cacheable: true,
            bufferable: true,
        });

        self
    }

    #[inline(always)]
    fn direct_mapped_section(&mut self, index: usize, section: L1Section) {
        assert!(index < L1_TABLE_SIZE);

        let base = (index as u32) << 20;
        self.table[index] = L1Entry::section(base, section);
    }
}


pub fn with_mmu<F: FnMut() -> !>(l1table: &L1Table, mut f: F) -> ! {
    let domains = AccessDomains::all_manager();
    DACR.write(domains.into());

    let table_base = &l1table.table as *const _ as u32;
    assert!(table_base & 0x3fff == 0);
    TTBR0.write(
        TTBR0::zeroed()
            .irgn1(true)
            .s(true)
            // Outer Cacheable Write-Back, no allocate on write.
            .rgn(0b11)
            .irgn0(true)
            .table_base(table_base >> 14)
    );

    // Enable I-Cache and D-Cache
    SCTLR.write(
        SCTLR::zeroed()
            .m(true)
            .a(false)
            .c(true)
            .i(true)
            .unaligned(true)
    );

    // Synchronization barriers
    // Allows MMU to start
    asm::dsb();
    // Flushes pre-fetch buffer
    asm::isb();

    f();
}

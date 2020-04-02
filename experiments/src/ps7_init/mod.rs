use libboard_zynq::println;

mod zc706;
// mod cora_z7_10;

#[cfg(feature = "target_zc706")]
use zc706 as target;
#[cfg(feature = "target_cora_z7_10")]
use cora_z7_10 as target;

pub fn report_differences() {
    for (i, op) in target::INIT_DATA.iter().enumerate() {
        let address = op.address();
        let overwritten_later = target::INIT_DATA[(i + 1)..].iter()
            .any(|later_op| later_op.address() == address);

        if !overwritten_later {
            op.report_difference();
        }
    }
}

pub enum InitOp {
    MaskWrite(usize, usize, usize),
    MaskPoll(usize, usize),
}

impl InitOp {
    fn address(&self) -> usize {
        match self {
            InitOp::MaskWrite(address, _, _) => *address,
            InitOp::MaskPoll(address, _) => *address,
        }
    }

    fn read(&self) -> usize {
        unsafe { *(self.address() as *const usize) }
    }

    fn difference(&self) -> Option<(usize, usize)> {
        let (mask, expected) = match self {
            InitOp::MaskWrite(_, mask, expected) =>
                (*mask, *expected),
            InitOp::MaskPoll(_, mask) =>
                (*mask, *mask),
        };
        let actual = self.read();
        if actual & mask == expected {
            None
        } else {
            Some((actual & mask, expected))
        }
    }

    pub fn report_difference(&self) {
        if let Some((actual, expected)) = self.difference() {
            println!(
                "Register {:08X} is {:08X}&={:08X} != {:08X} expected",
                self.address(),
                self.read(),
                actual,
                expected
            );
        }
    }
}

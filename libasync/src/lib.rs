#![no_std]

extern crate alloc;

mod delay;
pub mod executor;
pub mod task;
pub use delay::delay;

pub mod smoltcp;

/// Reexport for macro use
pub use nb;

/// The `nb` crate's `block!` macro adapted for async fns
///
/// Call `.await` on the result!
#[macro_export]
macro_rules! block_async {
    ($e:expr) => {
        async {
            loop {
                #[allow(unreachable_patterns)]
                match $e {
                    Err($crate::nb::Error::Other(e)) => {
                        #[allow(unreachable_code)]
                        break Err(e);
                    }
                    Err($crate::nb::Error::WouldBlock) => $crate::task::r#yield().await,
                    Ok(x) => break Ok(x),
                }
            }
        }
    };
}

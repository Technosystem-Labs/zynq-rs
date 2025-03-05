use alloc::{boxed::Box, vec::Vec};
use core::{
    cell::{RefCell, Cell},
    future::Future,
    mem::MaybeUninit,
    pin::{pin, Pin},
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

// NOTE `*const ()` is &AtomicBool
static VTABLE: RawWakerVTable = {
    unsafe fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    unsafe fn wake(p: *const ()) {
        wake_by_ref(p)
    }
    unsafe fn wake_by_ref(p: *const ()) {
        (*(p as *const AtomicBool)).store(true, Ordering::Relaxed)
    }
    unsafe fn drop(_: *const ()) {
        // no-op
    }

    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
};

/// ready should not move as long as this waker references it. That is
/// the reason for keeping Tasks in a pinned box.
fn wrap_waker(ready: &AtomicBool) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(ready as *const _ as *const (), &VTABLE)) }
}

/// A single-threaded executor
///
/// This is a singleton
pub struct Executor {
    // Entered block_on() already?
    in_block_on: Cell<bool>,

    /// Tasks reside on the heap, so that we just queue pointers. They
    /// must also be pinned in memory because our RawWaker is a pointer
    /// to their `ready` field.
    tasks: RefCell<Vec<Pin<Box<Task>>>>,
}

impl Executor {
    /// Creates a new instance of the executor
    pub fn new() -> Self {
        Self {
            in_block_on: Cell::new(false),
            tasks: RefCell::new(Vec::new()),
        }
    }

    pub fn block_on<T>(&self, f: impl Future<Output = T>) -> T {
        // we want to avoid reentering `block_on` because then all the code
        // below has to become more complex. It's also likely that the
        // application will only call `block_on` once on an infinite task
        // (`Future<Output = !>`)
        if self.in_block_on.get() {
            panic!("nested `block_on`");
        }
        self.in_block_on.replace(true);

        let mut pinned_f = pin!(f);
        let ready = AtomicBool::new(true);
        let waker = wrap_waker(&ready);
        let mut backup = Vec::new();
        let val = loop {
            // advance the main task
            if ready.load(Ordering::Relaxed) {
                ready.store(false, Ordering::Relaxed);
                let mut cx = Context::from_waker(&waker);
                if let Poll::Ready(val) = pinned_f.as_mut().poll(&mut cx) {
                    break val;
                }
            }

            // advance all tasks
            core::mem::swap(&mut *self.tasks.borrow_mut(), &mut backup);
            for mut task in backup.drain(..) {
                // NOTE we don't need a CAS operation here because `wake` invocations that come from
                // interrupt handlers (the only source of 'race conditions' (!= data races)) are
                // "oneshot": they'll issue a `wake` and then disable themselves to not run again
                // until the woken task has made more work
                if task.ready.load(Ordering::Relaxed) {
                    // we are about to service the task so switch the `ready` flag to `false`
                    task.ready.store(false, Ordering::Relaxed);

                    let waker = wrap_waker(&task.ready);
                    let mut cx = Context::from_waker(&waker);
                    let ready = task.f.as_mut().poll(&mut cx).is_ready();
                    if ready {
                        // Task is finished, do not requeue
                        continue;
                    }
                }
                // Requeue
                self.tasks.borrow_mut().push(task);
            }
        };
        self.in_block_on.replace(false);
        val
    }

    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        let task = Box::pin(Task::new(f));
        self.tasks.borrow_mut().push(task);
    }
}

pub struct Task {
    ready: AtomicBool,
    f: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    fn new(f: impl Future<Output = ()> + 'static) -> Self {
        Task {
            ready: AtomicBool::new(true),
            f: Box::pin(f),
        }
    }
}

/// Returns a handle to the executor singleton
///
/// This lazily initializes the executor and allocator when first called
pub(crate) fn current() -> &'static Executor {
    static INIT: AtomicBool = AtomicBool::new(false);
    static mut EXECUTOR: MaybeUninit<Executor> = MaybeUninit::uninit();

    if INIT.load(Ordering::Relaxed) {
        unsafe { EXECUTOR.assume_init_ref() }
    } else {
        unsafe {
            let executor = EXECUTOR.write(Executor::new());
            INIT.store(true, Ordering::Relaxed);
            &*executor
        }
    }
}

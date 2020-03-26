use core::{
    cell::{Cell, UnsafeCell},
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    sync::atomic::{self, AtomicBool, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use alloc::{boxed::Box, collections::VecDeque as Deque};
//use futures::future::FutureExt;
use pin_utils::pin_mut;
use libcortex_a9::mutex::Mutex;
// TODO: delete
use libboard_zynq::println;

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

/// A single-threaded executor
///
/// This is a singleton
pub struct Executor {
    in_block_on: Mutex<bool>,
    tasks: Mutex<Deque<Task>>,
}

impl Executor {
    /// Creates a new instance of the executor
    pub fn new() -> Self {
        Self {
            in_block_on: Mutex::new(false),
            tasks: Mutex::new(Deque::new()),
        }
    }

    pub fn block_on<T>(&self, f: impl Future<Output = T>) -> T {
        // we want to avoid reentering `block_on` because then all the code
        // below has to become more complex. It's also likely that the
        // application will only call `block_on` once on an infinite task
        // (`Future<Output = !>`)
        {
            let mut in_block_on = self.in_block_on.lock();
            if *in_block_on {
                panic!("nested `block_on`");
            }
            *in_block_on = true;
        }

        pin_mut!(f);
        let ready = AtomicBool::new(true);
        let waker =
            unsafe { Waker::from_raw(RawWaker::new(&ready as *const _ as *const _, &VTABLE)) };
        let val = loop {
            // advance the main task
            if ready.load(Ordering::Relaxed) {
                ready.store(false, Ordering::Relaxed);

                let mut cx = Context::from_waker(&waker);
                if let Poll::Ready(val) = f.as_mut().poll(&mut cx) {
                    break val;
                }
            }

            // advance other tasks
            let next_task = self.tasks.lock().pop_front();
            if let Some(mut task) = next_task {
                // NOTE we don't need a CAS operation here because `wake` invocations that come from
                // interrupt handlers (the only source of 'race conditions' (!= data races)) are
                // "oneshot": they'll issue a `wake` and then disable themselves to not run again
                // until the woken task has made more work
                if task.ready.load(Ordering::Relaxed) {
                    // we are about to service the task so switch the `ready` flag to `false`
                    task.ready.store(false, Ordering::Relaxed);

                    // NOTE we never deallocate tasks so `&ready` is always pointing to
                    // allocated memory (`&'static AtomicBool`)
                    let waker = unsafe {
                        Waker::from_raw(RawWaker::new(&task.ready as *const _ as *const _, &VTABLE))
                    };
                    let mut cx = Context::from_waker(&waker);
                    // this points into a `static` memory so it's already pinned
                    let r = unsafe {
                        Pin::new_unchecked(&mut *task.f)
                            .poll(&mut cx)
                            .is_ready()
                    };
                    if !r {
                        // Task is not finished, requeue
                        self.tasks.lock().push_back(task);
                    }
                }
            }

            // // try to sleep; this will be a no-op if any of the previous tasks generated a SEV or an
            // // interrupt ran (regardless of whether it generated a wake-up or not)
            // asm::wfe();
        };
        *self.in_block_on.lock() = false;
        val
    }

    // NOTE CAREFUL! this method can overlap with `block_on`
    // FIXME we want to use `Future<Output = !>` here but the never type (`!`) is unstable; so as a
    // workaround we'll "abort" if the task / future terminates (see `Task::new`)
    pub fn spawn(&self, f: impl Future + 'static) {
        // NOTE(unsafe) only safe as long as `spawn` is never re-entered and this does not overlap
        // with operation `(A)` (see `Task::block_on`)
        self.tasks.lock().push_back(Task::new(f));
    }
}

pub struct Task {
    ready: AtomicBool,
    f: Box<Future<Output = ()>>,
}

impl Task {
    fn new(f: impl Future + 'static) -> Self {
        Task {
            ready: AtomicBool::new(true),
            f: Box::new(async { f.await; }),
        }
    }
}

/// Returns a handle to the executor singleton
///
/// This lazily initializes the executor and allocator when first called
pub(crate) fn current() -> &'static Executor {
    static INIT: AtomicBool = AtomicBool::new(false);
    static mut EXECUTOR: UnsafeCell<MaybeUninit<Executor>> = UnsafeCell::new(MaybeUninit::uninit());

    if INIT.load(Ordering::Relaxed) {
        unsafe { &*(EXECUTOR.get() as *const Executor) }
    } else {
        unsafe {
            let executorp = EXECUTOR.get() as *mut Executor;
            executorp.write(Executor::new());
            INIT.store(true, Ordering::Relaxed);
            &*executorp
        }
    }
}

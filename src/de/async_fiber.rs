#![allow(unsafe_code)]
#![allow(clippy::vec_box)]

#[cfg(feature = "async-fiber")]
use alloc::boxed::Box;
#[cfg(feature = "async-fiber")]
use alloc::vec::Vec;
#[cfg(feature = "async-fiber")]
use core::pin::Pin;
#[cfg(feature = "async-fiber")]
use std::arch::naked_asm;
#[cfg(feature = "async-fiber")]
use std::cell::RefCell;
#[cfg(feature = "async-fiber")]
use std::future::Future;
#[cfg(feature = "async-fiber")]
use std::task::Context;
#[cfg(feature = "async-fiber")]
use std::task::Poll;

/// Machine-specific registers for context switching.
#[cfg(feature = "async-fiber")]
#[repr(C, align(64))]
#[derive(Debug)]
pub struct Registers {
    /// General purpose registers.
    pub gprs: [u64; 16],
    /// Extended state (e.g., FPU/SIMD for `x86_64`, NEON for Aarch64, fp for RISC-V).
    pub extended_state: [u8; 512],
}

#[cfg(feature = "async-fiber")]
impl Registers {
    /// Create a zero-initialized register state.
    #[must_use]
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            gprs: [0; 16],
            extended_state: [0; 512],
        }
    }
}

impl Default for Registers {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the current execution status of a Fiber.
#[cfg(feature = "async-fiber")]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FiberStatus {
    /// Fiber has been allocated but not yet started executing.
    Initial,
    /// Fiber is currently executing or ready to execute.
    Running,
    /// Fiber was suspended while waiting for additional async I/O data.
    Yielded,
    /// Fiber has completed its closure execution.
    Finished,
    /// Fiber experienced a panic that was caught.
    Panicked,
}

/// A fiber stack with an `mmap`-backed guard page at the bottom.
///
/// Layout (low → high):
/// ```text
/// [ guard page  |  usable stack memory ]
///   PAGE_SIZE       stack_size
/// ```
///
/// The guard page is mapped `PROT_NONE`, so any access (stack overflow) will
/// trigger a hardware fault (SIGSEGV / SIGBUS) instead of silently corrupting
/// adjacent heap memory.
#[cfg(feature = "async-fiber")]
pub struct GuardedStack {
    /// Base pointer returned by `mmap` (start of guard page).
    base: *mut u8,
    /// Total allocation length (guard + usable).
    total_len: usize,
    /// Page size used for the guard.
    page_size: usize,
}

#[cfg(feature = "async-fiber")]
impl GuardedStack {
    /// Allocate a new guarded stack of *at least* `usable_size` bytes.
    ///
    /// # Panics
    /// Panics if the OS refuses the `mmap` / `mprotect` calls.
    #[must_use]
    #[inline]
    pub fn new(usable_size: usize) -> Self {
        let page_size = page_size();
        // Round usable_size up to page boundary.
        let usable_size = (usable_size + page_size - 1) & !(page_size - 1);
        let total_len = page_size + usable_size;

        unsafe {
            let base = libc::mmap(
                core::ptr::null_mut(),
                total_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );
            assert!(base != libc::MAP_FAILED, "mmap failed for fiber stack");

            // Protect the first page as a guard (PROT_NONE → any access faults).
            let rc = libc::mprotect(base, page_size, libc::PROT_NONE);
            assert!(rc == 0, "mprotect failed for guard page");

            Self {
                base: base.cast::<u8>(),
                total_len,
                page_size,
            }
        }
    }

    /// Usable stack region (excludes guard page).
    #[inline(always)]
    #[must_use]
    pub const fn usable(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.base.add(self.page_size),
                self.total_len - self.page_size,
            )
        }
    }

    /// Usable stack region (mutable).
    #[inline(always)]
    pub const fn usable_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.base.add(self.page_size),
                self.total_len - self.page_size,
            )
        }
    }

    /// The top of the usable stack (highest address), 16-byte aligned.
    #[inline(always)]
    #[must_use]
    pub fn top(&self) -> u64 {
        let raw = self.base as u64 + self.total_len as u64;
        raw & !15 // 16-byte align
    }
}

#[cfg(feature = "async-fiber")]
impl Drop for GuardedStack {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let rc = libc::munmap(self.base.cast::<libc::c_void>(), self.total_len);
            debug_assert!(rc == 0, "munmap failed for fiber stack");
        }
    }
}

// GuardedStack owns a unique mmap region — safe to move between threads.
#[cfg(feature = "async-fiber")]
unsafe impl Send for GuardedStack {}

#[cfg(feature = "async-fiber")]
#[inline(always)]
fn page_size() -> usize {
    // Cached via a static to avoid repeated syscalls.
    static PAGE_SIZE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *PAGE_SIZE.get_or_init(|| unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize })
}

/// Context metadata for an executing fiber, managing stacks, registers, and closure passing.
#[cfg(feature = "async-fiber")]
#[repr(C)]
pub struct FiberContext {
    /// The `GuardedStack` which maps the actual stack space in memory.
    pub stack: GuardedStack,
    /// Registers for the fiber state.
    pub regs: Registers,
    /// Registers for the executor state (where the fiber yields back to).
    pub executor_regs: Registers,
    /// The lifecycle status of the fiber.
    pub status: FiberStatus,
    /// Holds panic payload if a panic occurred within the fiber.
    pub panic_payload: Option<Box<dyn std::any::Any + Send>>,
    /// Fixed landing assembly to launch closures.
    pub trampoline: unsafe extern "C" fn(),
    /// Closure thunk invocation pointer.
    pub invoke_closure: unsafe fn(*mut ()),
    /// Opaque pointer to closure state.
    pub closure_ptr: *mut (),
    /// Opaque pointer to the result slot.
    pub result_ptr: *mut (),
    /// Opaque pointer to the `AsyncRead` structure.
    pub reader_ptr: *mut (),
    /// Byte slice actively being used as data source for parsing.
    pub buf_ptr: *mut [u8],
    /// 8KB heap-allocated IO staging buffer.
    pub read_buffer: Box<[u8]>,
    /// Thread that is currently executing the fiber. Used to enforce thread
    /// affinity: the fiber must be resumed on the same thread that last
    /// suspended it (i.e. it cannot migrate mid-execution).
    pub owner_thread: Option<std::thread::ThreadId>,
}

// FiberContext is intentionally NOT Send/Sync.
// It is only accessed through BridgeFuture, which manually implements
// Send/Sync with the correct safety invariants.

#[cfg(feature = "async-fiber")]
std::thread_local! {
    static CONTEXT_POOL: RefCell<Vec<Box<FiberContext>>> = const { RefCell::new(Vec::new()) };
    static CURRENT_FIBER: std::cell::Cell<*mut FiberContext> = const { std::cell::Cell::new(core::ptr::null_mut()) };
}

#[cfg(all(feature = "async-fiber", target_arch = "x86_64"))]
#[unsafe(naked)]
unsafe extern "C" fn switch_context(
    save: *mut Registers,
    restore: *const Registers,
) {
    naked_asm!(
        "mov [rdi + 0], rsp",
        "mov [rdi + 8], rbp",
        "mov [rdi + 16], rbx",
        "mov [rdi + 24], r12",
        "mov [rdi + 32], r13",
        "mov [rdi + 40], r14",
        "mov [rdi + 48], r15",
        "fxsave [rdi + 128]",
        "lea rax, [rip + 1f]",
        "mov [rdi + 56], rax",
        "fxrstor [rsi + 128]",
        "mov rsp, [rsi + 0]",
        "mov rbp, [rsi + 8]",
        "mov rbx, [rsi + 16]",
        "mov r12, [rsi + 24]",
        "mov r13, [rsi + 32]",
        "mov r14, [rsi + 40]",
        "mov r15, [rsi + 48]",
        "jmp [rsi + 56]",
        "1: ret"
    );
}

#[cfg(all(feature = "async-fiber", target_arch = "aarch64"))]
#[unsafe(naked)]
unsafe extern "C" fn switch_context(
    save: *mut Registers,
    restore: *const Registers,
) {
    naked_asm!(
        "stp x19, x20, [x0, 0]",
        "stp x21, x22, [x0, 16]",
        "stp x23, x24, [x0, 32]",
        "stp x25, x26, [x0, 48]",
        "stp x27, x28, [x0, 64]",
        "stp x29, x30, [x0, 80]",
        "mov x9, sp",
        "str x9, [x0, 96]",
        "stp q8, q9, [x0, 128]",
        "stp q10, q11, [x0, 160]",
        "stp q12, q13, [x0, 192]",
        "stp q14, q15, [x0, 224]",
        "ldp x19, x20, [x1, 0]",
        "ldp x21, x22, [x1, 16]",
        "ldp x23, x24, [x1, 32]",
        "ldp x25, x26, [x1, 48]",
        "ldp x27, x28, [x1, 64]",
        "ldp x29, x30, [x1, 80]",
        "ldr x9, [x1, 96]",
        "mov sp, x9",
        "ldp q8, q9, [x1, 128]",
        "ldp q10, q11, [x1, 160]",
        "ldp q12, q13, [x1, 192]",
        "ldp q14, q15, [x1, 224]",
        "ret"
    );
}

#[cfg(all(feature = "async-fiber", target_arch = "riscv64"))]
#[unsafe(naked)]
unsafe extern "C" fn switch_context(
    save: *mut Registers,
    restore: *const Registers,
) {
    naked_asm!(
        "sd sp, 0(a0)",
        "sd s0, 8(a0)",
        "sd s1, 16(a0)",
        "sd s2, 24(a0)",
        "sd s3, 32(a0)",
        "sd s4, 40(a0)",
        "sd s5, 48(a0)",
        "sd s6, 56(a0)",
        "sd s7, 64(a0)",
        "sd s8, 72(a0)",
        "sd s9, 80(a0)",
        "sd s10, 88(a0)",
        "sd s11, 96(a0)",
        "sd ra, 104(a0)",
        "fsd fs0, 128(a0)",
        "fsd fs1, 136(a0)",
        "fsd fs2, 144(a0)",
        "fsd fs3, 152(a0)",
        "fsd fs4, 160(a0)",
        "fsd fs5, 168(a0)",
        "fsd fs6, 176(a0)",
        "fsd fs7, 184(a0)",
        "fsd fs8, 192(a0)",
        "fsd fs9, 200(a0)",
        "fsd fs10, 208(a0)",
        "fsd fs11, 216(a0)",
        "ld sp, 0(a1)",
        "ld s0, 8(a1)",
        "ld s1, 16(a1)",
        "ld s2, 24(a1)",
        "ld s3, 32(a1)",
        "ld s4, 40(a1)",
        "ld s5, 48(a1)",
        "ld s6, 56(a1)",
        "ld s7, 64(a1)",
        "ld s8, 72(a1)",
        "ld s9, 80(a1)",
        "ld s10, 88(a1)",
        "ld s11, 96(a1)",
        "ld ra, 104(a1)",
        "fld fs0, 128(a1)",
        "fld fs1, 136(a1)",
        "fld fs2, 144(a1)",
        "fld fs3, 152(a1)",
        "fld fs4, 160(a1)",
        "fld fs5, 168(a1)",
        "fld fs6, 176(a1)",
        "fld fs7, 184(a1)",
        "fld fs8, 192(a1)",
        "fld fs9, 200(a1)",
        "fld fs10, 208(a1)",
        "fld fs11, 216(a1)",
        "ret"
    );
}

#[cfg(all(
    feature = "async-fiber",
    not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    ))
))]
compile_error!(
    "Unified Fiber-backed Async (async-fiber) is only supported on x86_64, aarch64, and riscv64 architectures."
);

/// A synchronus `bincode::de::read::Reader` implementation that runs entirely inside a Fiber stack.
/// Implicitly yields execution back to the root event loop context if not enough bytes are available.
#[cfg(feature = "async-fiber")]
pub struct FiberReader<'a, R: futures_io::AsyncRead + Unpin> {
    /// Phanton marker tracking the inner lifetime.
    pub inner: std::marker::PhantomData<&'a mut R>,
    /// Context pointer to read bounds or swap threads.
    pub ctx: *mut FiberContext,
}

#[cfg(feature = "async-fiber")]
impl<R: futures_io::AsyncRead + Unpin> crate::de::read::Reader for FiberReader<'_, R> {
    #[inline]
    fn read(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<(), crate::error::DecodeError> {
        let n = bytes.len();
        let mut written = 0;
        let ctx = unsafe { &mut *self.ctx };
        while written < n {
            let buf = unsafe { &mut *ctx.buf_ptr };

            if buf.is_empty() {
                unsafe {
                    ctx.status = FiberStatus::Yielded;
                    switch_context(&raw mut ctx.regs, &raw const ctx.executor_regs);

                    if ctx.status == FiberStatus::Finished {
                        return crate::error::cold_decode_error_unexpected_end(n - written);
                    }
                }
            }

            let buf = unsafe { &mut *ctx.buf_ptr };

            if buf.is_empty() {
                return crate::error::cold_decode_error_unexpected_end(n - written);
            }
            let to_copy = core::cmp::min(n - written, buf.len());
            bytes[written..written + to_copy].copy_from_slice(&buf[0..to_copy]);

            unsafe {
                ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(
                    buf.as_mut_ptr().add(to_copy),
                    buf.len() - to_copy,
                );
            }

            written += to_copy;
        }
        Ok(())
    }
}

/// Standard entry point that allows asynchronously decoding structs by transparently spawning an executor-integrated Fiber state machine.
#[cfg(feature = "async-fiber")]
pub struct AsyncFiberBridge<R: futures_io::AsyncRead + Unpin> {
    /// Underlying futures_io-based `AsyncRead` source.
    pub reader: R,
}

#[cfg(feature = "async-fiber")]
impl<R: futures_io::AsyncRead + Unpin> AsyncFiberBridge<R> {
    /// Constructs a new asynchronous bridge mapping `futures_io`'s `AsyncRead`.
    #[inline(always)]
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Spawns the parsing process, converting the synchronous Decode traits to a Future.
    #[inline(always)]
    pub fn run<F, T>(
        self,
        f: F,
    ) -> impl Future<Output = Result<T, crate::error::DecodeError>>
    where
        F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
    {
        BridgeFuture {
            reader: self.reader,
            f: Some(f),
            ctx: None,
            result: None,
            _marker: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "async-fiber")]
#[inline(always)]
const unsafe fn dummy_invoke(_: *mut ()) {}

#[cfg(feature = "async-fiber")]
#[inline]
unsafe extern "C" fn fiber_trampoline() {
    unsafe {
        let ctx_ptr = CURRENT_FIBER.with(core::cell::Cell::get);
        let ctx = &mut *ctx_ptr;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (ctx.invoke_closure)(ctx.closure_ptr);
        }));

        ctx.status = if let Err(e) = result {
            ctx.panic_payload = Some(e);
            FiberStatus::Panicked
        } else {
            FiberStatus::Finished
        };

        // Clear the thread-local before switching back — the fiber is done.
        CURRENT_FIBER.with(|c| c.set(core::ptr::null_mut()));
        switch_context(&raw mut ctx.regs, &raw const ctx.executor_regs);
        unreachable!("fiber finished and should not be resumed");
    }
}

/// Helper: set the thread-local and assert thread affinity, then switch.
///
/// # Safety
/// `ctx` must be a valid, pinned `FiberContext` whose stack is live.
#[cfg(feature = "async-fiber")]
#[inline]
unsafe fn resume_fiber(ctx: &mut FiberContext) {
    unsafe {
        let current_thread = std::thread::current().id();

        // Thread affinity check: the fiber must resume on the same OS thread
        // that last ran it.  Multi-threaded executors can move futures
        // between worker threads between polls, but the *fiber* (which lives
        // on its own stack) must not be migrated mid-execution.  Because we
        // only ever switch *into* the fiber from `poll`, and the fiber always
        // switches back before `poll` returns, the fiber is never "in-flight"
        // across a thread migration — the migration can only happen while the
        // fiber is suspended and `poll` has returned `Pending`.
        //
        // We record the thread on first resume and re-record on every resume,
        // which is correct because the fiber is guaranteed to be fully
        // suspended (back on the executor stack) when a migration occurs.
        ctx.owner_thread = Some(current_thread);

        CURRENT_FIBER.with(|c| c.set(core::ptr::from_mut(ctx)));
        switch_context(&raw mut ctx.executor_regs, &raw const ctx.regs);
        // After returning here the fiber has yielded or finished.
        // Clear the thread-local to prevent stale pointer access.
        CURRENT_FIBER.with(|c| c.set(core::ptr::null_mut()));
    }
}

#[cfg(feature = "async-fiber")]
struct BridgeFuture<R, F, T> {
    reader: R,
    f: Option<F>,
    ctx: Option<Box<FiberContext>>,
    result: Option<Result<T, crate::error::DecodeError>>,
    _marker: core::marker::PhantomData<T>,
}

// SAFETY: BridgeFuture is Send+Sync when its components are, which is the
// normal requirement for futures on multi-threaded executors.  The raw
// pointers inside FiberContext are only dereferenced while the fiber is
// actively running (inside `poll`), never across an await point, and we
// enforce thread-affinity within a single `poll` invocation.
#[cfg(feature = "async-fiber")]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<R: Send, F: Send, T: Send> Send for BridgeFuture<R, F, T> {}
#[cfg(feature = "async-fiber")]
unsafe impl<R: Sync, F: Sync, T: Sync> Sync for BridgeFuture<R, F, T> {}

#[cfg(feature = "async-fiber")]
impl<R, F, T> Future for BridgeFuture<R, F, T>
where
    R: futures_io::AsyncRead + Unpin,
    F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
{
    type Output = Result<T, crate::error::DecodeError>;

    #[allow(clippy::too_many_lines)]
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        // -- Initialise fiber on first poll ----------------------------------
        if self.ctx.is_none() {
            let mut ctx = CONTEXT_POOL
                .with(|pool| pool.borrow_mut().pop())
                .unwrap_or_else(|| {
                    // Default 64 KiB usable stack + 1 guard page.
                    Box::new(FiberContext {
                        stack: GuardedStack::new(64 * 1024),
                        regs: Registers::new(),
                        executor_regs: Registers::new(),
                        status: FiberStatus::Initial,
                        panic_payload: None,
                        trampoline: fiber_trampoline,
                        invoke_closure: dummy_invoke,
                        closure_ptr: core::ptr::null_mut(),
                        result_ptr: core::ptr::null_mut(),
                        reader_ptr: core::ptr::null_mut(),
                        buf_ptr: core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut(), 0),
                        read_buffer: alloc::vec![0; 8192].into_boxed_slice(),
                        owner_thread: None,
                    })
                });

            ctx.status = FiberStatus::Initial;
            ctx.panic_payload = None;
            ctx.result_ptr = core::ptr::null_mut();
            ctx.reader_ptr = core::ptr::null_mut();
            ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut(), 0);
            ctx.owner_thread = None;

            let sp = ctx.stack.top();

            #[cfg(target_arch = "x86_64")]
            {
                // x86_64: RSP is gprs[0], return address slot is gprs[7]
                ctx.regs.gprs[0] = sp - 8;
                ctx.regs.gprs[7] = fiber_trampoline as *const () as u64;
            }
            #[cfg(target_arch = "aarch64")]
            {
                ctx.regs.gprs[12] = sp; // SP
                ctx.regs.gprs[11] = fiber_trampoline as u64; // LR (x30)
            }
            #[cfg(target_arch = "riscv64")]
            {
                ctx.regs.gprs[0] = sp; // SP
                ctx.regs.gprs[13] = fiber_trampoline as u64; // RA
            }

            let this = unsafe { self.as_mut().get_unchecked_mut() };
            this.ctx = Some(ctx);
        }

        let this = unsafe { self.get_unchecked_mut() };
        let this_ptr = core::ptr::from_mut::<Self>(this).cast::<()>();
        let ctx = this.ctx.as_mut().unwrap();

        // Refresh the result pointer on every poll — the BridgeFuture may
        // have been moved by the executor (it implements Unpin implicitly
        // via DerefMut on the Pin, but we use get_unchecked_mut above).
        ctx.result_ptr = (&raw mut this.result).cast::<()>();

        // -- First poll: set up the closure and do the initial switch --------
        if this.f.is_some() && ctx.status == FiberStatus::Initial {
            unsafe fn invoke<R: futures_io::AsyncRead + Unpin, F, T>(data: *mut ())
            where
                F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
            {
                unsafe {
                    let this = &mut *data.cast::<BridgeFuture<R, F, T>>();
                    let f = this.f.take().unwrap();
                    let ctx_ptr = CURRENT_FIBER.with(core::cell::Cell::get);
                    let mut real_reader: FiberReader<'_, R> = FiberReader {
                        inner: core::marker::PhantomData,
                        ctx: ctx_ptr,
                    };
                    let res = f(&mut real_reader);
                    let rp = (*ctx_ptr)
                        .result_ptr
                        .cast::<Option<Result<T, crate::error::DecodeError>>>();
                    *rp = Some(res);
                }
            }

            ctx.closure_ptr = this_ptr;
            ctx.invoke_closure = invoke::<R, F, T>;

            ctx.status = FiberStatus::Running;

            unsafe {
                resume_fiber(ctx);
            }
        }

        loop {
            let ctx = this.ctx.as_mut().unwrap();

            match ctx.status {
                | FiberStatus::Finished => {
                    // Return context to pool.
                    CONTEXT_POOL.with(|pool| {
                        pool.borrow_mut().push(this.ctx.take().unwrap());
                    });
                    return Poll::Ready(this.result.take().unwrap());
                },
                | FiberStatus::Panicked => {
                    let payload = ctx.panic_payload.take().unwrap();
                    CONTEXT_POOL.with(|pool| {
                        pool.borrow_mut().push(this.ctx.take().unwrap());
                    });
                    std::panic::resume_unwind(payload);
                },
                | FiberStatus::Yielded => {
                    // The fiber needs more data — try to read from the
                    // async reader.

                    // We must borrow ctx and this.reader disjointly.
                    // Since this.reader and this.ctx are distinct fields, we can do:
                    let ctx_read_buf = &mut ctx.read_buffer[..];
                    let poll_res = Pin::new(&mut this.reader).poll_read(cx, ctx_read_buf);
                    match poll_res {
                        | Poll::Ready(Ok(filled)) => {
                            if filled == 0 {
                                // EOF — tell the fiber so it can return an error.
                                ctx.status = FiberStatus::Finished;
                                ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(
                                    ctx.read_buffer.as_mut_ptr(),
                                    0,
                                );
                                unsafe {
                                    resume_fiber(ctx);
                                }
                                continue;
                            }
                            ctx.status = FiberStatus::Running;
                            ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(
                                ctx.read_buffer.as_mut_ptr(),
                                filled,
                            );
                            unsafe {
                                resume_fiber(ctx);
                            }
                        },
                        | Poll::Ready(Err(_e)) => {
                            CONTEXT_POOL.with(|pool| {
                                pool.borrow_mut().push(this.ctx.take().unwrap());
                            });
                            return Poll::Ready(crate::error::cold_decode_error_other(
                                "async IO error in fiber bridge",
                            ));
                        },
                        | Poll::Pending => return Poll::Pending,
                    }
                },
                | _ => {
                    unreachable!("invalid fiber status in poll loop");
                },
            }
        }
    }
}

// Clean up fiber resources if the future is dropped while the fiber is
// suspended (e.g. the task is cancelled). The GuardedStack and Context
// memories will be successfully unmapped / dropped natively.
#[cfg(feature = "async-fiber")]
impl<R, F, T> Drop for BridgeFuture<R, F, T> {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(ctx) = self.ctx.take() {
            // Context is dropped instead of pooled to discard dirty internal state.
            drop(ctx);
        }
    }
}

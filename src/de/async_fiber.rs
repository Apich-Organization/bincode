#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(unsafe_code)]

#[cfg(feature = "async-fiber")]
use alloc::boxed::Box;
#[cfg(feature = "async-fiber")]
use alloc::vec::Vec;
#[cfg(feature = "async-fiber")]
use core::pin::Pin;
#[cfg(feature = "async-fiber")]
use std::cell::RefCell;
#[cfg(feature = "async-fiber")]
use std::arch::naked_asm;
#[cfg(feature = "async-fiber")]
use std::future::Future;
#[cfg(feature = "async-fiber")]
use std::task::{Context, Poll};

#[cfg(feature = "async-fiber")]
#[repr(C, align(64))]
#[derive(Debug)]
pub struct Registers {
    pub gprs: [u64; 16],
    pub extended_state: [u8; 512],
}

#[cfg(feature = "async-fiber")]
impl Registers {
    pub const fn new() -> Self {
        Self {
            gprs: [0; 16],
            extended_state: [0; 512],
        }
    }
}

#[cfg(feature = "async-fiber")]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FiberStatus {
    Initial,
    Running,
    Yielded,
    Finished,
    Panicked,
}

#[cfg(feature = "async-fiber")]
#[repr(C)]
pub struct FiberContext {
    pub stack: Pin<Box<[u8]>>,
    pub regs: Registers,
    pub executor_regs: Registers,
    pub status: FiberStatus,
    pub panic_payload: Option<Box<dyn std::any::Any + Send>>,
    pub trampoline: unsafe extern "C" fn(),
    pub invoke_closure: unsafe fn(*mut ()),
    pub closure_ptr: *mut (),
    pub result_ptr: *mut (),
    pub reader_ptr: *mut (),
    pub buf_ptr: *mut [u8],
}
unsafe impl Send for FiberContext {}
unsafe impl Sync for FiberContext {}

#[cfg(feature = "async-fiber")]
std::thread_local! {
    static STACK_POOL: RefCell<Vec<Pin<Box<[u8]>>>> = RefCell::new(Vec::new());
    static CURRENT_FIBER: std::cell::Cell<*mut FiberContext> = std::cell::Cell::new(core::ptr::null_mut());
}

#[cfg(all(feature = "async-fiber", target_arch = "x86_64"))]
#[unsafe(naked)]
unsafe extern "C" fn switch_context(save: *mut Registers, restore: *const Registers) {
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
unsafe extern "C" fn switch_context(save: *mut Registers, restore: *const Registers) {
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
unsafe extern "C" fn switch_context(save: *mut Registers, restore: *const Registers) {
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

#[cfg(all(feature = "async-fiber", not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64"))))]
compile_error!("Unified Fiber-backed Async (async-fiber) is only supported on x86_64, aarch64, and riscv64 architectures.");

#[cfg(feature = "async-fiber")]
pub struct FiberReader<'a, R: tokio::io::AsyncRead + Unpin> {
    pub inner: std::marker::PhantomData<&'a mut R>,
    pub ctx: *mut FiberContext,
}

#[cfg(feature = "async-fiber")]
impl<'a, R: tokio::io::AsyncRead + Unpin> FiberReader<'a, R> {
    fn available(ctx: &mut FiberContext) -> usize {
        let buf = unsafe { &mut *ctx.buf_ptr };
        buf.len()
    }
}

#[cfg(feature = "async-fiber")]
impl<'a, R: tokio::io::AsyncRead + Unpin> crate::de::read::Reader for FiberReader<'a, R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), crate::error::DecodeError> {
        let n = bytes.len();
        let mut written = 0;
        let ctx = unsafe { &mut *self.ctx };
        while written < n {
            let buf = unsafe { &mut *ctx.buf_ptr };
            
            if buf.is_empty() {
                unsafe {
                    ctx.status = FiberStatus::Yielded;
                    switch_context(&mut ctx.regs, &ctx.executor_regs);
                    
                    if ctx.status == FiberStatus::Finished {
                        return Err(crate::error::DecodeError::UnexpectedEnd { additional: n - written });
                    }
                }
            }
            
            let buf = unsafe { &mut *ctx.buf_ptr };
            
            if !buf.is_empty() {
                let to_copy = core::cmp::min(n - written, buf.len());
                bytes[written..written + to_copy].copy_from_slice(&buf[0..to_copy]);
                
                // update buf_ptr using slice addressing
                unsafe {
                    ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(
                        buf.as_mut_ptr().add(to_copy),
                        buf.len() - to_copy
                    );
                }
                
                written += to_copy;
            } else {
                return Err(crate::error::DecodeError::UnexpectedEnd { additional: n - written });
            }
        }
        Ok(())
    }
}

#[cfg(feature = "async-fiber")]
pub struct AsyncFiberBridge<R: tokio::io::AsyncRead + Unpin> {
    pub reader: R,
}

#[cfg(feature = "async-fiber")]
impl<R: tokio::io::AsyncRead + Unpin> AsyncFiberBridge<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn run<F, T>(self, f: F) -> impl Future<Output = Result<T, crate::error::DecodeError>>
    where
        F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
    {
        BridgeFuture {
            reader: self.reader,
            f: Some(f),
            ctx: None,
            read_buffer: alloc::vec![0; 8192].into_boxed_slice(),
            valid_bytes: 0,
            consumed_bytes: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "async-fiber")]
pub struct DummyReader;

#[cfg(feature = "async-fiber")]
impl tokio::io::AsyncRead for DummyReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "async-fiber")]
unsafe fn dummy_invoke(_: *mut ()) {}

#[cfg(feature = "async-fiber")]
unsafe extern "C" fn fiber_trampoline() { unsafe {
    let ctx_ptr = CURRENT_FIBER.with(|c| c.get());
    let ctx = &mut *ctx_ptr;
    
    // We catch panic to handle unwind cleanly
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (ctx.invoke_closure)(ctx.closure_ptr);
    }));
    
    ctx.status = if result.is_err() {
        ctx.panic_payload = Some(result.unwrap_err());
        FiberStatus::Panicked
    } else {
        FiberStatus::Finished
    };
    
    switch_context(&mut ctx.regs, &ctx.executor_regs);
    unreachable!("fiber finished and should not be resumed");
}}

#[cfg(feature = "async-fiber")]
struct BridgeFuture<R, F, T> {
    reader: R,
    f: Option<F>,
    ctx: Option<Box<FiberContext>>,
    read_buffer: Box<[u8]>,
    valid_bytes: usize,
    consumed_bytes: usize,
    _marker: core::marker::PhantomData<T>,
}

#[cfg(feature = "async-fiber")]
impl<R, F, T> Future for BridgeFuture<R, F, T>
where
    R: tokio::io::AsyncRead + Unpin,
    F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
{
    type Output = Result<T, crate::error::DecodeError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
         if self.ctx.is_none() {
             let stack = STACK_POOL.with(|pool| pool.borrow_mut().pop()).unwrap_or_else(|| {
                 alloc::vec![0; 16384].into_boxed_slice().into()
             });
             
             let mut ctx = Box::new(FiberContext {
                 stack,
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
             });
             
             let sp = ctx.stack.as_ptr() as u64 + ctx.stack.len() as u64;
             let sp = sp & !15; // align to 16
             
             // Setup initial registers for each arch
             #[cfg(target_arch = "x86_64")]
             {
                 ctx.regs.gprs[0] = sp - 8;
                 ctx.regs.gprs[7] = fiber_trampoline as *const () as u64;
             }
             #[cfg(target_arch = "aarch64")]
             {
                 ctx.regs.gprs[12] = sp;
                 ctx.regs.gprs[11] = fiber_trampoline as u64;
             }
             #[cfg(target_arch = "riscv64")]
             {
                 ctx.regs.gprs[0] = sp;
                 ctx.regs.gprs[13] = fiber_trampoline as u64;
             }
             
             let this = unsafe { self.as_mut().get_unchecked_mut() };
             this.ctx = Some(ctx);
         }
         
         let this = unsafe { self.get_unchecked_mut() };
         let ctx = this.ctx.as_mut().unwrap();
         
         if let Some(f) = this.f.take() {
            let mut result_slot: Option<Result<T, crate::error::DecodeError>> = None;
            let result_ptr = &mut result_slot as *mut Option<Result<T, crate::error::DecodeError>>;
            ctx.result_ptr = result_ptr as *mut ();
            
            struct ClosureData<R: tokio::io::AsyncRead + Unpin, F, T> {
                f: F,
                _marker: core::marker::PhantomData<(R, T)>,
            }
            
            unsafe fn invoke<R: tokio::io::AsyncRead + Unpin, F, T>(data: *mut ()) 
            where
                F: FnOnce(&mut FiberReader<'_, R>) -> Result<T, crate::error::DecodeError>,
            { unsafe {
                let data = Box::from_raw(data as *mut ClosureData<R, F, T>);
                let f = data.f;
                let ctx_ptr = CURRENT_FIBER.with(|c| c.get());
                let mut real_reader: FiberReader<'_, R> = FiberReader {
                    inner: core::marker::PhantomData,
                    ctx: ctx_ptr,
                };
                let res = f(&mut real_reader);
                let rp = (*ctx_ptr).result_ptr as *mut Option<Result<T, crate::error::DecodeError>>;
                *rp = Some(res);
            }}

            let data = Box::new(ClosureData::<R, F, T> { f, _marker: core::marker::PhantomData });
            ctx.closure_ptr = Box::into_raw(data) as *mut ();
            ctx.invoke_closure = invoke::<R, F, T>;
            
            ctx.status = FiberStatus::Running;
            
            CURRENT_FIBER.with(|c| c.set(&mut **ctx as *mut _));
            unsafe {
                switch_context(&mut ctx.executor_regs, &ctx.regs);
            }
         }
         
         loop {
             let ctx = this.ctx.as_mut().unwrap();
             
             if ctx.status == FiberStatus::Finished {
                 let res = unsafe {
                     let rp = ctx.result_ptr as *mut Option<Result<T, crate::error::DecodeError>>;
                     (*rp).take().unwrap()
                 };
                 // Return stack to pool
                 STACK_POOL.with(|pool| pool.borrow_mut().push(this.ctx.take().unwrap().stack));
                 return Poll::Ready(res);
             } else if ctx.status == FiberStatus::Panicked {
                 let payload = ctx.panic_payload.take().unwrap();
                 STACK_POOL.with(|pool| pool.borrow_mut().push(this.ctx.take().unwrap().stack));
                 std::panic::resume_unwind(payload);
             } else if ctx.status == FiberStatus::Yielded {
                 // Try reading more data
                 let mut buf = tokio::io::ReadBuf::new(&mut this.read_buffer[..]);
                 let poll_res = Pin::new(&mut this.reader).poll_read(cx, &mut buf);
                 match poll_res {
                     Poll::Ready(Ok(())) => {
                         let filled = buf.filled().len();
                         if filled == 0 {
                             ctx.status = FiberStatus::Finished; // EOF
                             CURRENT_FIBER.with(|c| c.set(&mut **ctx as *mut _));
                             // switch to let fiber notice EOF and error out
                             ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(this.read_buffer.as_mut_ptr(), 0);
                             unsafe {
                                 switch_context(&mut ctx.executor_regs, &ctx.regs);
                             }
                             continue;
                         } else {
                             ctx.status = FiberStatus::Running;
                             ctx.buf_ptr = core::ptr::slice_from_raw_parts_mut(this.read_buffer.as_mut_ptr(), filled);
                             CURRENT_FIBER.with(|c| c.set(&mut **ctx as *mut _));
                             unsafe {
                                 switch_context(&mut ctx.executor_regs, &ctx.regs);
                             }
                             continue;
                         }
                     },
                     Poll::Ready(Err(_e)) => {
                         STACK_POOL.with(|pool| pool.borrow_mut().push(this.ctx.take().unwrap().stack));
                         return Poll::Ready(Err(crate::error::DecodeError::Other("IO error")));
                     },
                     Poll::Pending => return Poll::Pending,
                 }
             } else {
                 unreachable!("Invalid fiber status");
             }
         }
    }
}

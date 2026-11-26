//! # 栈式协程与上下文切换（riscv64）
//!
//! 在本练习中，你用内联汇编实现最小化的上下文切换，
//! 这是操作系统线程调度的核心机制。这个 crate **只支持 riscv64**；
//! 请在 riscv64 Linux 上运行 `cargo test`，或在 x86 上通过 QEMU 走仓库的常规流程（`./check.sh` / `oscamp`）。
//!
//! ## 关键概念
//! - **被调用者保存寄存器**：切换时保存并恢复它们，被切走的任务之后才能正确恢复执行。
//! - **栈指针 `sp`** 与 **返回地址 `ra`**：在新上下文中恢复它们；第一次切换到某个任务时，`ret` 会跳转到 `ra`（入口点）。
//! - 内联汇编：`core::arch::asm!`
//!
//! ## riscv64 ABI（本练习）
//! - 被调用者保存：`sp`、`ra`、`s0`–`s11`。`ret` 指令即 `jalr zero, 0(ra)`。
//! - 第一、第二个参数：`a0`（旧上下文）、`a1`（新上下文）。

#![cfg(target_arch = "riscv64")]

/// 一个任务的寄存器保存状态（riscv64）。布局必须与下面汇编使用的偏移量一致：
/// `sp` 在 0，`ra` 在 8，随后 `s0`–`s11` 分别在 16、24、…… 104。
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskContext {
    pub sp: u64,
    pub ra: u64,
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    pub s8: u64,
    pub s9: u64,
    pub s10: u64,
    pub s11: u64,
}

impl TaskContext {
    pub const fn empty() -> Self {
        Self {
            sp: 0,
            ra: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }

    /// 初始化该上下文，使我们切换到它时从 `entry` 开始执行。
    ///
    /// - 设置 `ra = entry`，这样新上下文里的第一次 `ret` 会跳转到 `entry`。
    /// - 设置 `sp = stack_top`，并按 16 字节对齐（RISC-V ABI 要求函数入口处的栈 16 字节对齐）。
    /// - `s0`–`s11` 保持为 0；它们会在切换时被加载。
    pub fn init(&mut self, stack_top: usize, entry: usize) {
        self.ra = entry as u64;
        self.sp = (stack_top & !0xF) as u64;
    }
}

/// 从 `old` 上下文切换到 `new`：把当前被调用者保存寄存器存入 `old`，从 `new` 加载，然后 `ret`（跳转到 `new.ra`）。
///
/// 汇编层面：把 `sp`、`ra`、`s0`–`s11` 存入 `[a0]`（旧上下文），从 `[a1]`（新上下文）加载，并把 `a0`/`a1` 清零，以免把指针泄漏到新上下文，最后执行 `ret`。
///
/// 必须标注 `#[unsafe(naked)]`，防止编译器生成序言/尾声。
#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(_old: &mut TaskContext, _new: &TaskContext) {
    core::arch::naked_asm!(
        // 保存旧上下文的被调用者保存寄存器到 [a0]
        "sd sp,   0(a0)",
        "sd ra,   8(a0)",
        "sd s0,  16(a0)",
        "sd s1,  24(a0)",
        "sd s2,  32(a0)",
        "sd s3,  40(a0)",
        "sd s4,  48(a0)",
        "sd s5,  56(a0)",
        "sd s6,  64(a0)",
        "sd s7,  72(a0)",
        "sd s8,  80(a0)",
        "sd s9,  88(a0)",
        "sd s10, 96(a0)",
        "sd s11, 104(a0)",
        // 从 [a1] 加载新上下文的被调用者保存寄存器
        "ld sp,   0(a1)",
        "ld ra,   8(a1)",
        "ld s0,  16(a1)",
        "ld s1,  24(a1)",
        "ld s2,  32(a1)",
        "ld s3,  40(a1)",
        "ld s4,  48(a1)",
        "ld s5,  56(a1)",
        "ld s6,  64(a1)",
        "ld s7,  72(a1)",
        "ld s8,  80(a1)",
        "ld s9,  88(a1)",
        "ld s10, 96(a1)",
        "ld s11, 104(a1)",
        // 清空参数寄存器，避免将上下文指针泄漏到新上下文
        "li a0, 0",
        "li a1, 0",
        // 跳转到 new.ra
        "ret",
    );
}

const STACK_SIZE: usize = 1024 * 64;

/// 为协程分配一个栈。返回 `(buffer, stack_top)`，其中 `stack_top` 是高地址
/// （栈向下增长）。只要使用该栈的上下文还存在，buffer 就必须保持存活。
pub fn alloc_stack() -> (Vec<u8>, usize) {
    let buf = vec![0u8; STACK_SIZE];
    let top = buf.as_ptr() as usize + STACK_SIZE;
    (buf, top)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    extern "C" fn task_entry() {
        COUNTER.store(42, Ordering::SeqCst);
        loop {
            std::hint::spin_loop();
        }
    }

    #[test]
    fn test_alloc_stack() {
        let (buf, top) = alloc_stack();
        assert_eq!(top, buf.as_ptr() as usize + STACK_SIZE);
        assert!(top % 16 == 0);
    }

    #[test]
    fn test_context_init() {
        let (buf, top) = alloc_stack();
        let _ = buf;
        let mut ctx = TaskContext::empty();
        let entry = task_entry as *const () as usize;
        ctx.init(top, entry);
        assert_eq!(ctx.ra, entry as u64);
        assert!(ctx.sp != 0);
    }

    #[test]
    fn test_switch_to_task() {
        COUNTER.store(0, Ordering::SeqCst);

        static mut MAIN_CTX_PTR: *mut TaskContext = std::ptr::null_mut();
        static mut TASK_CTX_PTR: *mut TaskContext = std::ptr::null_mut();

        extern "C" fn cooperative_task() {
            COUNTER.store(99, Ordering::SeqCst);
            unsafe {
                switch_context(&mut *TASK_CTX_PTR, &*MAIN_CTX_PTR);
            }
        }

        let (_stack_buf, stack_top) = alloc_stack();
        let mut main_ctx = TaskContext::empty();
        let mut task_ctx = TaskContext::empty();
        task_ctx.init(stack_top, cooperative_task as *const () as usize);

        unsafe {
            MAIN_CTX_PTR = &mut main_ctx;
            TASK_CTX_PTR = &mut task_ctx;
            switch_context(&mut main_ctx, &task_ctx);
        }

        assert_eq!(COUNTER.load(Ordering::SeqCst), 99);
    }
}

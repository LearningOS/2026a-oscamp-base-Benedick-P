//! # 读写锁（写者优先）
//!
//! 在本练习中，你将用原子操作从零实现一个**写者优先**的读写锁。
//! 多个读者可以同时持有锁；写者则独占锁。
//!
//! **注意：** Rust 标准库已经提供了 [`std::sync::RwLock`]。本练习为了学习
//! 协议与策略，实现一个不使用标准库的最小版本。
//!
//! ## 读写锁的常见策略
//! 当读者和写者同时在等待时，不同实现的**优先级**不同：
//!
//! - **读者优先**：写者在等待时仍允许新读者进入，若读者不断到来，
//!   写者可能被饿死。
//! - **写者优先**：一旦有写者在等待，就不再接纳新读者，直到该写者
//!   运行完毕；本练习实现的是这种策略。
//! - **读写公平**：请求按公平顺序处理（例如 FIFO 或轮转），
//!   读者和写者都不会被系统性饿死。
//!
//! ## 关键概念
//! - **读者**：共享访问；多个线程可以同时持有读锁。
//! - **写者**：独占访问；只能有一个写者，且写者持锁期间没有读者。
//! - **写者优先（本实现）**：只要至少有一个写者在等待，新读者就会被阻塞，
//!   直到该写者运行。
//!
//! ## 状态（单个原子变量）
//! 我们用一个 `AtomicU32`：低位 = 读者计数，两个标志位 = 写者持有 / 写者等待。
//! 全部逻辑都用 compare_exchange 和 load/store 实现，不使用 `std::sync::RwLock`。

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};

/// 并发读者的最大数量（能放进状态位）。
const READER_MASK: u32 = (1 << 30) - 1;
/// 写者持有锁时置位的标志。
const WRITER_HOLDING: u32 = 1 << 30;
/// 至少有一个写者在等待时置位的标志（写者优先：阻止新读者）。
const WRITER_WAITING: u32 = 1 << 31;

/// 写者优先的读写锁。从零实现，不使用 `std::sync::RwLock`。
pub struct RwLock<T> {
    state: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// 获取读锁。自旋阻塞，直到没有写者持有且没有写者在等待（写者优先）。
    ///
    /// TODO: 实现读锁获取
    /// 1. 循环中读取 state（Acquire）。
    /// 2. 若置位了 WRITER_HOLDING 或 WRITER_WAITING，则 spin_loop 后继续（写者优先：写者等待期间不接纳新读者）。
    /// 3. 若读者计数（state & READER_MASK）已达 READER_MASK，则自旋后继续。
    /// 4. 尝试 compare_exchange(s, s + 1, AcqRel, Acquire)；成功后返回 RwLockReadGuard { lock: self }。
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        // TODO
        loop {
            let s = self.state.load(Ordering::Acquire);
    
            // ① 写者持有或写者在等 → 自旋（写者优先）
            if s & (WRITER_HOLDING | WRITER_WAITING) != 0 {
                std::hint::spin_loop();
                continue;
            }
    
            // ② 读者计数已满
            if s & READER_MASK == READER_MASK {
                std::hint::spin_loop();
                continue;
            }
    
            // ③ 尝试把读者计数 +1
            if self.state
                .compare_exchange(s, s + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return RwLockReadGuard { lock: self };
            }
            std::hint::spin_loop();
        }
    }

    /// 获取写锁。阻塞直到没有读者且没有其他写者。
    ///
    /// TODO: 实现写锁获取（写者优先）
    /// 1. 先设置 WRITER_WAITING：fetch_or(WRITER_WAITING, Release)，使新读者被阻塞。
    /// 2. 循环：读取 state；若存在读者（READER_MASK）或 WRITER_HOLDING，则 spin_loop 后继续。
    /// 3. 尝试 compare_exchange(WRITER_WAITING, WRITER_HOLDING, ...) 获取锁；若写者刚释放，也可尝试 compare_exchange(0, WRITER_HOLDING, ...)。
    /// 4. 成功后返回 RwLockWriteGuard { lock: self }。
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        // TODO
        loop {
            let s = self.state.load(Ordering::Acquire);
    
            // ① 确保 WAITING 置位：挡住新读者
            if s & WRITER_WAITING == 0 {
                self.state.fetch_or(WRITER_WAITING, Ordering::Release);
                continue;
            }
    
            // ② 还有读者或别的写者 → 等
            if s & (READER_MASK | WRITER_HOLDING) != 0 {
                std::hint::spin_loop();
                continue;
            }
    
            // ③ WAITING → HOLDING
            if self.state
                .compare_exchange(
                    s,
                    (s & !WRITER_WAITING) | WRITER_HOLDING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return RwLockWriteGuard { lock: self };
            }
            std::hint::spin_loop();
        }
    }
}

/// 读锁守卫；在 drop 时释放读锁。
pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

// TODO: 为 RwLockReadGuard 实现 Deref
// 返回数据的共享引用：unsafe { &*self.lock.data.get() }
impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

// TODO: 为 RwLockReadGuard 实现 Drop
// 读者计数减一：self.lock.state.fetch_sub(1, Ordering::Release)
impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

/// 写锁守卫；在 drop 时释放写锁。
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

// TODO: 为 RwLockWriteGuard 实现 Deref
// 返回共享引用：unsafe { &*self.lock.data.get() }
impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

// TODO: 为 RwLockWriteGuard 实现 DerefMut
// 返回可变引用：unsafe { &mut *self.lock.data.get() }
impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

// TODO: 为 RwLockWriteGuard 实现 Drop
// 清除写者标志位以释放锁：self.lock.state.fetch_and(!(WRITER_HOLDING | WRITER_WAITING), Ordering::Release)
impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .state
            .fetch_and(!(WRITER_HOLDING | WRITER_WAITING), Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_multiple_readers() {
        let lock = Arc::new(RwLock::new(0u32));
        let mut handles = vec![];
        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let g = l.read();
                assert_eq!(*g, 0);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_writer_excludes_readers() {
        let lock = Arc::new(RwLock::new(0u32));
        let lock_w = Arc::clone(&lock);
        let writer = thread::spawn(move || {
            let mut g = lock_w.write();
            *g = 42;
        });
        writer.join().unwrap();
        let g = lock.read();
        assert_eq!(*g, 42);
    }

    #[test]
    fn test_concurrent_reads_after_write() {
        let lock = Arc::new(RwLock::new(Vec::<i32>::new()));
        {
            let mut g = lock.write();
            g.push(1);
            g.push(2);
        }
        let mut handles = vec![];
        for _ in 0..5 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let g = l.read();
                assert_eq!(g.len(), 2);
                assert_eq!(&*g, &[1, 2]);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_writes_serialized() {
        let lock = Arc::new(RwLock::new(0u64));
        let mut handles = vec![];
        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let mut g = l.write();
                    *g += 1;
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(*lock.read(), 1000);
    }
}

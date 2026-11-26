//! # 自旋锁
//!
//! 在本练习中，你将实现一个基础的自旋锁。
//! 自旋锁是操作系统内核中最基础的同步原语之一。
//!
//! ## 关键概念
//! - 自旋锁通过忙等待获取锁
//! - 用 `AtomicBool` 的 `compare_exchange` 实现加锁
//! - 用 `core::hint::spin_loop` 降低 CPU 功耗
//! - `UnsafeCell` 提供内部可变性

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// 基础自旋锁
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// 获取锁，返回内部数据的可变引用。
    ///
    /// TODO: 用 compare_exchange 自旋直到获得锁
    /// 1. 在循环中尝试把 locked 从 false 改为 true
    /// 2. 成功用 Acquire 内存序，失败用 Relaxed
    /// 3. 失败时调用 `core::hint::spin_loop()` 提示 CPU
    /// 4. 成功时返回 `&mut *self.data.get()`
    ///
    /// # 安全性
    /// 调用者必须保证用完数据后调用 `unlock`。
    pub fn lock(&self) -> &mut T {
        // TODO
        loop {
            if self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return unsafe { &mut *self.data.get() };
            }
            std::hint::spin_loop();
        }
    }

    /// 释放锁。
    ///
    /// TODO: 把 locked 设为 false（使用 Release 内存序）
    pub fn unlock(&self) {
        // TODO
        self.locked.store(false, Ordering::Release);
    }

    /// 尝试获取锁，不自旋。
    /// 成功返回 Some(&mut T)，锁被占用时返回 None。
    pub fn try_lock(&self) -> Option<&mut T> {
        // TODO: 只尝试一次 compare_exchange
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(unsafe { &mut *self.data.get() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_lock_unlock() {
        let lock = SpinLock::new(0u32);
        {
            let data = lock.lock();
            *data = 42;
            lock.unlock();
        }
        let data = lock.lock();
        assert_eq!(*data, 42);
        lock.unlock();
    }

    #[test]
    fn test_try_lock() {
        let lock = SpinLock::new(0u32);
        assert!(lock.try_lock().is_some());
        lock.unlock();
    }

    #[test]
    fn test_concurrent_counter() {
        let lock = Arc::new(SpinLock::new(0u64));
        let mut handles = vec![];

        for _ in 0..10 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let data = l.lock();
                    *data += 1;
                    l.unlock();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        assert_eq!(*data, 10000);
        lock.unlock();
    }

    #[test]
    fn test_lock_protects_data() {
        let lock = Arc::new(SpinLock::new(Vec::new()));
        let mut handles = vec![];

        for i in 0..5 {
            let l = Arc::clone(&lock);
            handles.push(thread::spawn(move || {
                let data = l.lock();
                data.push(i);
                l.unlock();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let data = lock.lock();
        let mut sorted = data.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2, 3, 4]);
        lock.unlock();
    }
}

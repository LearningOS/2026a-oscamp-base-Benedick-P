//! # 原子操作基础
//!
//! 在本练习中，你将使用原子类型实现一个无锁的线程安全计数器。
//!
//! ## 关键概念
//! - `std::sync::atomic::AtomicU64`
//! - `fetch_add`、`fetch_sub`、`load`、`store` 操作
//! - `compare_exchange` 无锁原语
//! - `Ordering` 内存序

use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    pub const fn new(init: u64) -> Self {
        Self {
            value: AtomicU64::new(init),
        }
    }

    /// 原子地加 1，返回**自增前**的值。
    ///
    /// 提示：使用 `fetch_add`，内存序用 `Ordering::Relaxed`
    pub fn increment(&self) -> u64 {
        // TODO
        self.value.fetch_add(1, Ordering::Relaxed)
    }

    /// 原子地减 1，返回**自减前**的值。
    pub fn decrement(&self) -> u64 {
        // TODO
        self.value.fetch_sub(1, Ordering::Relaxed)
    }

    /// 获取当前值。
    pub fn get(&self) -> u64 {
        // TODO
        self.value.load(Ordering::Relaxed)
    }

    /// 原子 CAS（比较并交换）操作。
    /// 若当前值等于 `expected`，则设为 `new_val` 并返回 Ok(expected)。
    /// 否则返回 Err(实际的当前值)。
    ///
    /// 提示：使用 `compare_exchange`，成功时内存序 `Ordering::AcqRel`，失败时 `Ordering::Acquire`
    pub fn compare_and_swap(&self, expected: u64, new_val: u64) -> Result<u64, u64> {
        // TODO
        self.value
            .compare_exchange(expected, new_val, Ordering::AcqRel, Ordering::Acquire)
    }

    /// 用 CAS 循环原子地做乘法。
    /// 返回**乘法前**的值。
    ///
    /// 提示：循环读取当前值，算出新值，尝试 CAS 更新，失败则重试。
    pub fn fetch_multiply(&self, multiplier: u64) -> u64 {
        // TODO: CAS 循环
        // loop {
        //     let current = ...
        //     let new = current * multiplier;
        //     match self.compare_and_swap(current, new) { ... }
        // }
        let mut current = self.value.load(Ordering::Relaxed);
        loop {
            let new = current.wrapping_mul(multiplier);
            match self.value.compare_exchange_weak(
                current, new, Ordering::AcqRel, Ordering::Acquire,
            ) {
                Ok(_) => return current,
                Err(actual) => current = actual,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_ops() {
        let c = AtomicCounter::new(0);
        assert_eq!(c.increment(), 0);
        assert_eq!(c.increment(), 1);
        assert_eq!(c.get(), 2);
        assert_eq!(c.decrement(), 2);
        assert_eq!(c.get(), 1);
    }

    #[test]
    fn test_cas_success() {
        let c = AtomicCounter::new(10);
        assert_eq!(c.compare_and_swap(10, 20), Ok(10));
        assert_eq!(c.get(), 20);
    }

    #[test]
    fn test_cas_failure() {
        let c = AtomicCounter::new(10);
        assert_eq!(c.compare_and_swap(5, 20), Err(10));
        assert_eq!(c.get(), 10);
    }

    #[test]
    fn test_fetch_multiply() {
        let c = AtomicCounter::new(3);
        let old = c.fetch_multiply(4);
        assert_eq!(old, 3);
        assert_eq!(c.get(), 12);
    }

    #[test]
    fn test_concurrent_increment() {
        let counter = Arc::new(AtomicCounter::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let c = Arc::clone(&counter);
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    c.increment();
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(counter.get(), 10000);
    }
}

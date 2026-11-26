//! # 互斥锁共享状态
//!
//! 在本练习中，你将使用 `Arc<Mutex<T>>` 在多个线程之间安全地共享并修改数据。
//!
//! ## 相关概念
//! - `Mutex<T>` 互斥锁用于保护共享数据
//! - `Arc<T>` 原子引用计数支持跨线程共享
//! - `lock()` 获取锁并访问数据

use std::sync::{Arc, Mutex};
use std::thread;

/// 使用 `n_threads` 个线程并发地递增同一个计数器。
/// 每个线程把计数器递增 `count_per_thread` 次。
/// 返回计数器的最终值。
///
/// 提示：用 `Arc<Mutex<usize>>` 作为共享计数器。
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize {
    // TODO: 创建初值为 0 的 Arc<Mutex<usize>>
    // TODO: 启动 n_threads 个线程
    // TODO: 每个线程调用 lock() 并递增 count_per_thread 次
    // TODO: 等待所有线程结束，返回最终值
    let counter = Arc::new(Mutex::new(0usize));
    let mut handles = Vec::with_capacity(n_threads);

    for _ in 0..n_threads {
        let counter = Arc::clone(&counter);   // 每个线程一份 Arc 所有权
        handles.push(thread::spawn(move || {
            for _ in 0..count_per_thread {
                let mut guard = counter.lock().unwrap(); // 拿锁
                *guard += 1;                             // 临界区
            }                                            // guard 离开作用域 → 自动解锁
        }));
    }

    for handle in handles {
        handle.join().unwrap();               // 等所有线程结束
    }

    let total = *counter.lock().unwrap();     // 读终值
    total
}

/// 用多个线程并发地向同一个共享向量添加元素。
/// 每个线程把自己的 id（0..n_threads）压入向量。
/// 返回排序后的向量。
///
/// 提示：使用 `Arc<Mutex<Vec<usize>>>`。
pub fn concurrent_collect(n_threads: usize) -> Vec<usize> {
    // TODO: 创建 Arc<Mutex<Vec<usize>>>
    // TODO: 每个线程压入自己的 id
    // TODO: 等待所有线程结束后排序并返回
     let shared = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::with_capacity(n_threads);

    for id in 0..n_threads {
        let shared = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            shared.lock().unwrap().push(id);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut result = Arc::try_unwrap(shared)
        .expect("所有线程已结束，主线程是唯一持有者")
        .into_inner()
        .expect("锁未被毒化");
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_single_thread() {
        assert_eq!(concurrent_counter(1, 100), 100);
    }

    #[test]
    fn test_counter_multi_thread() {
        assert_eq!(concurrent_counter(10, 100), 1000);
    }

    #[test]
    fn test_counter_zero() {
        assert_eq!(concurrent_counter(5, 0), 0);
    }

    #[test]
    fn test_collect() {
        let result = concurrent_collect(5);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_collect_single() {
        assert_eq!(concurrent_collect(1), vec![0]);
    }
}

//! # Tokio 异步任务
//!
//! 在本练习中，你将使用 `tokio::spawn` 创建并发的异步任务。
//!
//! ## 相关概念
//! - `tokio::spawn` 创建异步任务
//! - `JoinHandle` 等待任务完成
//! - 异步任务之间的并发执行

use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// 并发计算 0..n 中每个数的平方，按顺序收集结果并返回。
///
/// 提示：为每个 i 创建 `tokio::spawn` 任务，收集 JoinHandle，再依次 await。
pub async fn concurrent_squares(n: usize) -> Vec<usize> {
    // TODO: 创建 n 个异步任务，每个计算 i * i
    // TODO: 收集所有 JoinHandle
    // TODO: 逐个 await 取得结果
    let mut handles: Vec<JoinHandle<usize>> = Vec::with_capacity(n);
    for i in 0..n {
        handles.push(tokio::spawn(async move { i * i }));
    }
    let mut results = Vec::with_capacity(n);
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// 并发执行多个“耗时”任务（用 sleep 模拟），返回所有结果。
/// 每个任务休眠 `duration_ms` 毫秒，然后返回自己的 `task_id`。
///
/// 关键：所有任务应并发执行，总耗时应接近单个任务的耗时，而不是所有任务耗时之和。
pub async fn parallel_sleep_tasks(n: usize, duration_ms: u64) -> Vec<usize> {
    // TODO: 为 0..n 中的每个 id 创建异步任务
    // TODO: 每个任务休眠指定时长并返回自己的 id
    // TODO: 收集所有结果并排序
    let mut handles: Vec<JoinHandle<usize>> = Vec::with_capacity(n);
    for id in 0..n {
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(duration_ms)).await;
            id
        }));
    }
    let mut results = Vec::with_capacity(n);
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results.sort();
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_squares_basic() {
        let result = concurrent_squares(5).await;
        assert_eq!(result, vec![0, 1, 4, 9, 16]);
    }

    #[tokio::test]
    async fn test_squares_zero() {
        let result = concurrent_squares(0).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_squares_one() {
        let result = concurrent_squares(1).await;
        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn test_parallel_sleep() {
        let start = Instant::now();
        let result = parallel_sleep_tasks(5, 100).await;
        let elapsed = start.elapsed();

        assert_eq!(result, vec![0, 1, 2, 3, 4]);
        // 并发执行，总时间应远小于 5 * 100ms
        assert!(
            elapsed.as_millis() < 400,
            "Tasks should run concurrently, took {}ms",
            elapsed.as_millis()
        );
    }
}

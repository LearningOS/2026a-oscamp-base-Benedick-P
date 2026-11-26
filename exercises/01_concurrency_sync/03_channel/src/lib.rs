//! # 通道通信
//!
//! 在本练习中，你将使用 `std::sync::mpsc` 通道在线程之间传递消息。
//!
//! ## 相关概念
//! - `mpsc::channel()` 创建一个多生产者、单消费者通道
//! - `Sender::send()` 发送消息
//! - `Receiver::recv()` 接收消息
//! - 可以通过 `Sender::clone()` 创建多个生产者

use std::sync::mpsc;
use std::thread;

/// 创建一个生产者线程，把 items 中的每个元素发送到通道。
/// 主线程接收所有消息并返回它们。
pub fn simple_send_recv(items: Vec<String>) -> Vec<String> {
    // TODO: 创建通道
    // TODO: 启动线程，发送 items 中的每个元素
    // TODO: 在主线程中接收所有消息并收集到 Vec
    // 提示：所有 Sender 都被丢弃后，recv() 会返回 Err
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        for item in items {
            tx.send(item).unwrap();
        }
    });   // 闭包结束，tx 在这里 drop → 通道关闭

    let mut received = Vec::new();
    for msg in rx {          // recv 直到所有 Sender 都 drop
        received.push(msg);
    }

    handle.join().unwrap();
    received
}

/// 创建 `n_producers` 个生产者线程，每个线程发送一条格式为 `"msg from {id}"` 的消息。
/// 收集所有消息，按字典序排序后返回。
///
/// 提示：用 `tx.clone()` 创建多个发送端。注意原始的 tx 也必须丢弃。
pub fn multi_producer(n_producers: usize) -> Vec<String> {
    // TODO: 创建通道
    // TODO: 为每个生产者克隆一个发送端
    // TODO: 记得丢弃原始发送端，否则接收端不会结束
    // TODO: 收集所有消息并排序
    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::with_capacity(n_producers);
    for id in 0..n_producers {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            tx.send(format!("msg from {id}")).unwrap();
        }));
    }

    drop(tx);   // 关键：原始发送端也要丢弃

    let mut messages: Vec<String> = rx.iter().collect();

    for handle in handles {
        handle.join().unwrap();
    }

    messages.sort();
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_send_recv() {
        let items = vec!["hello".into(), "world".into(), "rust".into()];
        let result = simple_send_recv(items.clone());
        assert_eq!(result, items);
    }

    #[test]
    fn test_simple_empty() {
        let result = simple_send_recv(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_multi_producer() {
        let result = multi_producer(3);
        assert_eq!(
            result,
            vec![
                "msg from 0".to_string(),
                "msg from 1".to_string(),
                "msg from 2".to_string(),
            ]
        );
    }

    #[test]
    fn test_multi_producer_single() {
        let result = multi_producer(1);
        assert_eq!(result, vec!["msg from 0".to_string()]);
    }
}

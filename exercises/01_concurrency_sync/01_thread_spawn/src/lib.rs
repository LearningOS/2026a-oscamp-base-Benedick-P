//! # 线程创建
//!
//! 在本练习中，你将学习如何创建线程，以及如何在线程之间传递数据。
//!
//! ## 相关概念
//! - `std::thread::spawn` 创建一个新线程
//! - `move` 闭包捕获变量的所有权
//! - `JoinHandle::join()` 等待线程结束并取回返回值
//!
//! ## 进阶线程操作
//! - **线程休眠**：`thread::sleep` 让当前线程暂停执行。
//! - **线程局部存储**：`thread_local!` 宏定义每个线程独有的静态变量。
//! - **线程命名**：`Builder::name` 为线程设置名字，便于调试。
//! - **线程优先级**：通过 `thread::Builder` 设置（取决于平台）。
//! - **线程池**：`rayon` 等库负责管理线程复用。
//! - **线程通信**：使用 `std::sync::mpsc`（多生产者单消费者）或第三方 crate（如 `crossbeam`）。
//! - **共享状态**：`Arc<Mutex<T>>` 或 `Arc<RwLock<T>>` 可在线程之间安全共享可变数据。
//! - **同步原语**：`Barrier` 同步多个线程，`Condvar` 实现条件变量。
//! - **线程 park/unpark**：`thread::park` 阻塞线程，`unpark` 将其唤醒，适用于自定义调度。
//! - **获取当前线程句柄**：`thread::current()`。
//! - **作用域线程**：`crossbeam::scope` 或标准库的 `thread::scope`（Rust 1.63+）允许线程借用栈上数据，无需 `move`。
//!
//! Rust 通过所有权系统以及 `Send` 和 `Sync` trait 在编译期阻止数据竞争。
//! 实现了 `Send` 的类型可以跨线程边界转移。
//! 实现了 `Sync` 的类型可以被多个线程同时引用。
//! 大多数 Rust 标准类型都实现了 `Send + Sync`；例外包括 `Rc<T>`（非原子引用计数）和裸指针。
//!
//! ## 练习结构
//! 1. **基础练习**（`double_in_thread`、`parallel_sum`）——介绍最基本的线程创建。
//! 2. **进阶练习**（`named_sleeper`、`increment_thread_local`、`scoped_slice_sum`、`handle_panic`）——探索更多线程操作。
//! 每个函数都带有 `TODO` 注释，标明你需要编写代码的位置。
//! 运行 `cargo test` 检查你的实现。

#[allow(unused_imports)]
use std::cell::RefCell;
#[allow(unused_imports)]
use std::thread;
#[allow(unused_imports)]
use std::time::Duration;

// ============================================================================
// 示例代码：进阶线程模式
// ============================================================================
// 下面的示例演示了与线程相关的其他概念，
// 这些概念在真实的 Rust 并发编程中很有用。

/// 示例：处理线程 panic。
///
/// `join()` 返回 `Result`。如果线程发生 panic，返回的就是 `Err`。
/// 这里演示如何捕获并处理由子线程引发的 panic。
///
/// ```rust
/// use std::thread;
///
/// fn panic_handling_example() {
///     let handle = thread::spawn(|| {
///         // Simulate a panic
///         panic!("Thread panicked!");
///     });
///
///     match handle.join() {
///         Ok(_) => println!("Thread completed successfully."),
///         Err(e) => println!("Thread panicked: {:?}", e),
///     }
/// }
/// ```
///
/// 相比之下，下面的练习为了简洁使用 `unwrap()`，
/// 并假定线程不会 panic。

/// 示例：命名线程与自定义栈大小。
///
/// 使用 `thread::Builder` 可以给线程指定名字（便于调试），
/// 并设置它的栈大小。
///
/// ```rust
/// use std::thread;
///
/// fn named_thread_example() {
///     let builder = thread::Builder::new()
///         .name("my-worker".into())
///         .stack_size(32 * 1024); // 32 KiB
///
///     let handle = builder.spawn(|| {
///         println!("Hello from thread: {:?}", thread::current().name());
///         42
///     }).unwrap();
///
///     let result = handle.join().unwrap();
///     println!("Thread returned: {}", result);
/// }
/// ```

/// 示例：作用域线程（Rust 1.63+）。
///
/// 作用域线程允许在不转移所有权的情况下借用栈上数据。
/// 这些线程保证在作用域结束前全部结束，因此引用
/// 始终有效。
///
/// ```rust
/// use std::thread;
///
/// fn scoped_thread_example() {
///     let a = vec![1, 2, 3];
///     let b = vec![4, 5, 6];
///
///     let (sum_a, sum_b) = thread::scope(|s| {
///         let h1 = s.spawn(|| a.iter().sum::<i32>());
///         let h2 = s.spawn(|| b.iter().sum::<i32>());
///         (h1.join().unwrap(), h2.join().unwrap())
///     });
///
///     // `a` and `b` are still accessible here.
///     println!("sum_a = {}, sum_b = {}", sum_a, sum_b);
/// }
/// ```

/// 示例：线程局部存储。
///
/// 每个线程都会得到 `thread_local!` 变量的独立副本。
///
/// ```rust
/// use std::cell::RefCell;
/// use std::thread;
///
/// thread_local! {
///     static THREAD_ID: RefCell<usize> = RefCell::new(0);
/// }
///
/// fn thread_local_example() {
///     THREAD_ID.with(|id| {
///         *id.borrow_mut() = 1;
///     });
///
///     let handle = thread::spawn(|| {
///         THREAD_ID.with(|id| {
///             *id.borrow_mut() = 2;
///         });
///         THREAD_ID.with(|id| println!("Thread local value: {}", *id.borrow()));
///     });
///
///     handle.join().unwrap();
///
///     THREAD_ID.with(|id| println!("Main thread value: {}", *id.borrow()));
/// }
/// ```

// ============================================================================
// 练习函数
// ============================================================================

/// 在新线程中把向量里的每个元素乘以 2，返回结果向量。
///
/// 提示：使用 `thread::spawn` 和 `move` 闭包。
#[allow(unused_variables)]
pub fn double_in_thread(numbers: Vec<i32>) -> Vec<i32> {
    // TODO: 创建一个新线程，把 numbers 的每个元素乘以 2
    // 使用 thread::spawn 和 move 闭包
    // 使用 join().unwrap() 取回结果
    let handle =thread::spawn(move ||{
        numbers.iter().map(|x| x * 2).collect::<Vec<i32>>()
    });
    handle.join().unwrap()
}

/// 并行地对两个向量求和，返回两个和组成的元组。
///
/// 提示：为每个向量各创建一个线程。
#[allow(unused_variables)]
pub fn parallel_sum(a: Vec<i32>, b: Vec<i32>) -> (i32, i32) {
    // TODO: 创建两个线程，分别对 a 和 b 求和
    // 等待两个线程结束并取回结果
    let a_sum= thread::spawn(move||{
        a.iter().sum::<i32>()
    });
   
    let b_sum = thread::spawn(move||{
        b.iter().sum::<i32>()
    });
    (a_sum.join().unwrap(), b_sum.join().unwrap())
}

// ============================================================================
// 进阶练习函数
// ============================================================================

/// 创建一个命名线程，让它休眠指定的毫秒数，然后返回传入的值。
///
/// 线程名应为 `"sleeper"`。使用 `thread::Builder` 设置名字。
/// 在线程内部，先调用 `thread::sleep(Duration::from_millis(ms))`，再返回 `value`。
///
/// 提示：`thread::sleep` 只会阻塞当前线程，不影响其他线程。
#[allow(unused_variables)]
pub fn named_sleeper(value: i32, ms: u64) -> i32 {
    // TODO: 创建一个名为 "sleeper" 的线程构造器
    // TODO: 启动线程，休眠 `ms` 毫秒后返回 `value`
    // TODO: 等待线程结束并返回该值
    thread::Builder::new().name("sleeper".into())
    .spawn(move || {
        thread::sleep(Duration::from_millis(ms));
        value
    })
    .unwrap()
    .join()
    .unwrap()
}

thread_local! {
    static THREAD_COUNT: RefCell<usize> = RefCell::new(0);
}

/// 使用线程局部存储统计每个线程调用 `increment` 的次数。
///
/// 定义一个 `thread_local!` 静态变量 `THREAD_COUNT`，类型为 `RefCell<usize>`，初值为 0。
/// 每次调用 `increment` 都应把本线程的计数加 1，并返回新值。
///
/// 提示：用 `THREAD_COUNT.with(|cell| { ... })` 访问线程局部变量。
pub fn increment_thread_local() -> usize {
    // TODO: 使用 THREAD_COUNT.with 自增并返回新的计数
    THREAD_COUNT.with(|cell| {
        let mut n = cell.borrow_mut();
        *n += 1;
        *n
    })
}

/// 用**作用域线程**同时启动两个线程，在不转移所有权的情况下计算两个切片之和。
///
/// 使用 `thread::scope` 让线程借用切片 `&[i32]`。
/// 每个线程计算自己那份切片的和，函数返回 `(sum_a, sum_b)`。
///
/// 提示：切片是引用，无法 move 进闭包。
/// `thread::scope` 保证所有启动的线程在作用域结束前完成，
/// 因此借用是安全的。
#[allow(unused_variables)]
pub fn scoped_slice_sum(a: &[i32], b: &[i32]) -> (i32, i32) {
    // TODO: 使用 thread::scope 启动两个线程
    // TODO: 每个线程对自己的切片求和
    // TODO: 等待两个线程结束并返回结果
    thread::scope(|s| {
        let h1 = s.spawn(|| a.iter().sum::<i32>());
        let h2 = s.spawn(|| b.iter().sum::<i32>());
        (h1.join().unwrap(), h2.join().unwrap())
    })
}

/// 处理子线程可能发生的 panic。
///
/// 启动一个可能 panic 的线程：若 `should_panic` 为 `true`，线程调用 `panic!("oops")`；
/// 否则返回 `value`。
/// 线程正常结束时函数返回 `Ok(value)`，
/// 线程 panic 时返回 `Err(())`。
///
/// 提示：`join()` 返回 `Result<Result<i32, Box<dyn Any + Send>>, _>`。
/// 你需要同时匹配外层的 `Result`（线程是否 panic）和内层的 `Result`（线程是否返回了 `Result`）。
/// 本练习中内层类型就是 `i32`，不是 `Result`。
#[allow(unused_variables)]
pub fn handle_panic(value: i32, should_panic: bool) -> Result<i32, ()> {
    // TODO: 启动一个线程，让它 panic 或返回 value
    // TODO: join 并按需映射结果
    let h = thread::spawn(move || {
        if should_panic {
            panic!("oops");
        }
        value
    });
    h.join().map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_basic() {
        let nums = vec![1, 2, 3, 4, 5];
        assert_eq!(double_in_thread(nums), vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_double_empty() {
        assert_eq!(double_in_thread(vec![]), vec![]);
    }

    #[test]
    fn test_double_negative() {
        assert_eq!(double_in_thread(vec![-1, 0, 1]), vec![-2, 0, 2]);
    }

    #[test]
    fn test_parallel_sum() {
        let a = vec![1, 2, 3];
        let b = vec![10, 20, 30];
        assert_eq!(parallel_sum(a, b), (6, 60));
    }

    #[test]
    fn test_parallel_sum_empty() {
        assert_eq!(parallel_sum(vec![], vec![]), (0, 0));
    }

    // 进阶练习测试
    #[test]
    fn test_named_sleeper() {
        // 该线程只会休眠很短时间；这里只验证它返回正确的值。
        let result = named_sleeper(42, 10); // sleep 10 ms
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_local() {
        // 每个线程都有各自的计数器，因此启动两个线程并分别调用 increment，
        // 应该得到各自独立的序列。
        use std::sync::Arc;
        use std::sync::Mutex;

        let counters = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let counters = Arc::clone(&counters);
            handles.push(thread::spawn(move || {
                let v1 = increment_thread_local();
                let v2 = increment_thread_local();
                counters.lock().unwrap().push((v1, v2));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let results = counters.lock().unwrap();
        // 每个线程都应独立计数得到 (1, 2)。
        assert_eq!(results.len(), 2);
        assert!(results.contains(&(1, 2)));
    }

    #[test]
    fn test_scoped_slice_sum() {
        let a = [1, 2, 3];
        let b = [10, 20, 30];
        let (sum_a, sum_b) = scoped_slice_sum(&a, &b);
        assert_eq!(sum_a, 6);
        assert_eq!(sum_b, 60);
        // 确认切片仍可访问（它们是被借用，而不是被移动）。
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_handle_panic_ok() {
        let result = handle_panic(100, false);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn test_handle_panic_error() {
        let result = handle_panic(100, true);
        assert_eq!(result, Err(()));
    }
}

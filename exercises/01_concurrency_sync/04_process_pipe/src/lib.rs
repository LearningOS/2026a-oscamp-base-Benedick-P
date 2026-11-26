//! # 进程与管道
//!
//! 在本练习中，你将学习如何创建子进程，并通过管道与它通信。
//!
//! ## 相关概念
//! - `std::process::Command` 创建子进程（对应 `fork()` + `execve()` 系统调用）
//! - `Stdio::piped()` 建立管道（对应 `pipe()` + `dup2()` 系统调用）
//! - 通过 stdin/stdout 与子进程通信
//! - 获取子进程的退出状态（对应 `waitpid()` 系统调用）
//!
//! ## 与操作系统概念的对应
//! 本练习演示了用户态对底层操作系统原语的抽象：
//! - **进程创建**：Rust 的 `Command::new()` 内部调用 `fork()` 创建子进程，
//!   再调用 `execve()`（或等价调用）用目标程序替换子进程的内存映像。
//! - **进程间通信（IPC）**：管道是由内核管理的缓冲区，允许相关进程之间进行单向
//!   数据流动。`pipe()` 系统调用创建一个管道，返回两个文件描述符
//!   （读端、写端）。`dup2()` 复制文件描述符，从而实现
//!   标准输入/输出的重定向。
//! - **资源管理**：文件描述符（包括管道两端）会在其 Rust `Stdio` 对象
//!   被丢弃时自动关闭，避免资源泄漏。
//!
//! ## 练习结构
//! 1. **基础命令执行**（`run_command`）——启动子进程并捕获它的 stdout。
//! 2. **双向管道通信**（`pipe_through_cat`）——向子进程（`cat`）发送数据
//!    并读取它的输出。
//! 3. **获取退出码**（`get_exit_code`）——取得子进程的终止状态。
//! 4. **进阶：带错误处理的版本**（`run_command_with_result`）——学习正确的错误传播方式。
//! 5. **进阶：复杂的双向通信**（`pipe_through_grep`）——与一个过滤程序交互，
//!    它读取多行输入并输出筛选后的结果。
//!
//! 每个函数都带有 `TODO` 注释，标明你需要编写代码的位置。
//! 运行 `cargo test` 检查你的实现。

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

/// 执行给定的 shell 命令并返回它的 stdout 输出。
///
/// 例如：`run_command("echo", &["hello"])` 应返回 `"hello\n"`
///
/// # 底层系统调用
/// - `Command::new(program)` → `fork()` + `execve()` 系列
/// - `Stdio::piped()` → `pipe()` + `dup2()`（为 stdout 建立管道）
/// - `.output()` → `waitpid()`（等待子进程终止）
///
/// # 实现步骤
/// 1. 用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())` 以捕获子进程的 stdout。
/// 3. 调用 `.output()` 执行子进程并取得它的 `Output`。
/// 4. 把 `stdout` 字段（`Vec<u8>`）转换为 `String`。
pub fn run_command(program: &str, args: &[&str]) -> String {
    // TODO: 使用 Command::new 创建进程
    // TODO: 把 stdout 设置为 Stdio::piped()
    // TODO: 用 .output() 执行并取得输出
    // TODO: 把 stdout 转成 String 并返回
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("命令执行失败");
    String::from_utf8(output.stdout).expect("输出不是合法 UTF-8")
}

/// 通过管道把数据写入子进程（cat）的 stdin，并读取它的 stdout 输出。
///
/// 这里演示父子进程之间的双向管道通信。
///
/// # 底层系统调用
/// - `Command::new("cat")` → `fork()` + `execve("cat")`
/// - `Stdio::piped()`（调用两次）→ `pipe()` 创建两个管道（stdin 与 stdout）+ `dup2()` 完成重定向
/// - `ChildStdin::write_all()` → 向管道的写端执行 `write()`
/// - `drop(stdin)` → 关闭写端，向子进程发送 EOF
/// - `ChildStdout::read_to_string()` → 从管道读端执行 `read()`
///
/// # 所有权与资源管理
/// Rust 的所有权系统保证管道在正确的时机关闭：
/// 1. `ChildStdin` 句柄归父进程所有；向它写入会把数据传给子进程。
/// 2. 写完之后显式 `drop(stdin)`（或让它离开作用域）以关闭写端。
/// 3. 关闭写端会向 `cat` 发送 EOF，使它处理完所有输入后退出。
/// 4. 随后把 `ChildStdout` 读到结尾；丢弃它会关闭读端。
///
/// 如果不丢弃 `stdin`，子进程会一直等待更多输入（管道永远不会关闭）。
///
/// # 实现步骤
/// 1. 为 `"cat"` 创建 `Command`，设置 `.stdin(Stdio::piped())` 和 `.stdout(Stdio::piped())`。
/// 2. 调用 `.spawn()` 启动命令，得到一个带有 `stdin` 和 `stdout` 句柄的 `Child`。
/// 3. 把 `input` 字节写入子进程的 stdin（`child.stdin.take().unwrap().write_all(...)`）。
/// 4. 丢弃 stdin 句柄（显式 `drop` 或让它离开作用域）以关闭管道。
/// 5. 读取子进程的 stdout（`child.stdout.take().unwrap().read_to_string(...)`）。
/// 6. 用 `.wait()` 等待子进程退出（或依赖 drop 时自动等待）。
pub fn pipe_through_cat(input: &str) -> String {
    // TODO: 创建 "cat" 命令，把 stdin 和 stdout 设为 piped
    // TODO: 启动进程
    // TODO: 把 input 写入子进程的 stdin
    // TODO: 丢弃 stdin 以关闭管道（否则 cat 不会退出）
    // TODO: 从子进程的 stdout 读取输出
    let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("无法启动 cat");

// 临时 ChildStdin 在分号处 drop → 关闭写端 → cat 读到 EOF
child
    .stdin
    .take()
    .unwrap()
    .write_all(input.as_bytes())
    .unwrap();

let mut output = String::new();
child
    .stdout
    .take()
    .unwrap()
    .read_to_string(&mut output)
    .unwrap();
child.wait().unwrap();
output
}

/// 获取子进程的退出码。
/// 执行命令 `sh -c {command}` 并返回退出码。
///
/// # 底层系统调用
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`
/// - `.args(["-c", command])` 传入 shell 命令行
/// - `.status()` → `waitpid()`（等待子进程并取得退出状态）
/// - `ExitStatus::code()` 提取低字节的退出码（0-255）
///
/// # 实现步骤
/// 1. 为 `"sh"` 创建 `Command`，参数为 `["-c", command]`。
/// 2. 调用 `.status()` 执行 shell 并获得 `ExitStatus`。
/// 3. 用 `.code()` 取得 `Option<i32>` 形式的退出码。
/// 4. 若子进程正常终止，返回其退出码；否则返回一个默认值。
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: 使用 Command::new("sh").args(["-c", command])
    // TODO: 执行并取得状态
    // TODO: 返回退出码
    let status = Command::new("sh")
    .args(["-c", command])
    .status()
    .expect("无法执行 sh");
status.code().unwrap_or(-1)
}

/// 执行给定的 shell 命令，并以 `Result` 返回它的 stdout 输出。
///
// 这个版本会正确传播进程创建、执行或 I/O 过程中可能出现的错误
/// （例如命令不存在、权限不足、管道破裂）。
///
/// # 底层系统调用
/// 与 `run_command` 相同，但会捕获来自操作系统的错误并以 `Err` 返回。
///
/// # 错误处理
/// - `Command::new()` 只是构造构建器；错误发生在 `.output()` 时。
/// - `.output()` 返回 `Result<Output, std::io::Error>`。
/// - 如果子进程输出不是合法的 UTF-8，`String::from_utf8()` 可能失败。
///   这种情况下我们返回 kind 为 `InvalidData` 的 `io::Error`。
///
/// # 实现步骤
/// 1. 用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())`。
/// 3. 调用 `.output()` 并传播任何 `io::Error`。
/// 4. 用 `String::from_utf8` 把 `stdout` 转成 `String`；若失败则映射为 `io::Error`。
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: 使用 Command::new 创建进程
    // TODO: 把 stdout 设置为 Stdio::piped()
    // TODO: 用 .output() 执行并处理 Result
    // TODO: 用 from_utf8 把 stdout 转成 String，并把错误映射为 io::Error
    let output = Command::new(program)
    .args(args)
    .stdout(Stdio::piped())
    .output()?;
String::from_utf8(output.stdout)
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// 通过双向管道与 `grep` 交互，筛选出包含指定模式的行。
///
/// 这里演示复杂的父子进程通信：父进程发送多行输入，
/// 子进程（`grep`）按模式筛选，
/// 父进程只读回匹配的行。
///
/// # 底层系统调用
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - 与 `pipe_through_cat` 一样使用两个管道（stdin 与 stdout）
/// - 逐行写入与读取，模拟交互式过滤
///
/// # 实现步骤
/// 1. 为 `"grep"` 创建 `Command`，参数为 `pattern`，两端都设为 piped。
/// 2. 调用 `.spawn()` 启动命令，得到带有 `stdin` 和 `stdout` 句柄的 `Child`。
/// 3. 把 `input` 的每一行（以 `'\n'` 分隔）写入子进程的 stdin。
/// 4. 关闭写端（丢弃 stdin）以发送 EOF。
/// 5. 逐行读取子进程的 stdout，收集匹配的行。
/// 6. 等待子进程退出（可选；`grep` 在 EOF 后就会退出）。
/// 7. 把匹配行拼接成一个 `String` 返回。
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: 创建 "grep" 命令并带上 pattern，把 stdin 和 stdout 设为 piped
    // TODO: 启动进程
    // TODO: 把 input 的各行写入子进程 stdin
    // TODO: 丢弃 stdin 以关闭管道
    // TODO: 逐行读取子进程 stdout 的输出
    // TODO: 收集并返回匹配的行
    let mut child = Command::new("grep")
        .arg(pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("无法启动 grep");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    child.wait().unwrap();
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let output = run_command("echo", &["hello"]);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_run_with_args() {
        let output = run_command("echo", &["-n", "no newline"]);
        assert_eq!(output, "no newline");
    }

    #[test]
    fn test_pipe_cat() {
        let output = pipe_through_cat("hello pipe!");
        assert_eq!(output, "hello pipe!");
    }

    #[test]
    fn test_pipe_multiline() {
        let input = "line1\nline2\nline3";
        assert_eq!(pipe_through_cat(input), input);
    }

    #[test]
    fn test_exit_code_success() {
        assert_eq!(get_exit_code("true"), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        assert_eq!(get_exit_code("false"), 1);
    }

    #[test]
    fn test_run_command_with_result_success() {
        let result = run_command_with_result("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_run_command_with_result_nonexistent() {
        let result = run_command_with_result("nonexistent_command_xyz", &[]);
        // 命令不存在，应该报错
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_through_grep_basic() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("a", input);
        // grep 输出匹配的行并带上换行
        assert_eq!(output, "apple\nbanana\n");
    }

    #[test]
    fn test_pipe_through_grep_no_match() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("z", input);
        // 没有行匹配 -> 空字符串
        assert_eq!(output, "");
    }

    #[test]
    fn test_pipe_through_grep_multiline() {
        let input = "first line\nsecond line\nthird line\n";
        let output = pipe_through_grep("second", input);
        assert_eq!(output, "second line\n");
    }
}

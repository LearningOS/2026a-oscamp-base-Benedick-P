//! # no_std 内存原语
//!
//! 在 `#![no_std]` 环境中没有标准库，只有 `core`。
//! 这些内存操作函数是操作系统内核中最基础的构件。
//! 在裸机环境中，libc 里的 memcpy/memset 之类必须由我们自己实现。
//!
//! ## 任务
//!
//! 实现下面五个函数：
//! - 只使用 `core` crate，不使用 `std`
//! - 不要调用 `core::ptr::copy`、`core::ptr::copy_nonoverlapping` 等（自己写循环）
//! - 正确处理边界情况（n=0、内存区域重叠等）
//! - 通过全部测试

// 生产环境强制 no_std；测试时允许 std（cargo test 框架需要）
#![cfg_attr(not(test), no_std)]
#![allow(unused_variables)]

/// 从 `src` 复制 `n` 个字节到 `dst`。
///
/// - `dst` 与 `src` 不能重叠（重叠时请用 `my_memmove`）
/// - 返回 `dst`
///
/// # 安全性
/// `dst` 和 `src` 都必须各指向至少 `n` 字节的合法内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memcpy
    // 提示：从 src 逐字节读出并写入 dst
   
        let mut i = 0;
        while i < n {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
            i += 1;
        }
        dst
    
}

/// 把从 `dst` 起的 `n` 个字节设为值 `c`。
///
/// 返回 `dst`。
///
/// # 安全性
/// `dst` 必须指向至少 `n` 字节的可写合法内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memset(dst: *mut u8, c: u8, n: usize) -> *mut u8 {
    // TODO: 实现 memset
    let mut i = 0;
    while i < n {
        unsafe {
            *dst.add(i) = c;
        }
        i += 1;
    }
    dst
}

/// 从 `src` 复制 `n` 个字节到 `dst`，并正确处理内存重叠。
///
/// 返回 `dst`。
///
/// # 安全性
/// `dst` 和 `src` 都必须各指向至少 `n` 字节的合法内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memmove
    // 提示：当 dst > src 且区域重叠时，从后往前复制
    let d = dst as usize;
    let s = src as usize;

    if d <= s || d - s >= n {
        // 不重叠，或 dst 在 src 前面：正向复制
        let mut i = 0;
        while i < n {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
            i += 1;
        }
        
    } else {
        // dst 在 src 后面且重叠：必须从后往前
        let mut i = n;
        while i > 0 {
            i -= 1;
            unsafe {
                *dst.add(i) = *src.add(i);
            }
        }
    }
    dst
}

/// 返回以空字节结尾的字节串长度，不包含结尾的空字节。
///
/// # 安全性
/// `s` 必须指向合法的、以空字节结尾的字节串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    // TODO: 实现 strlen
    let mut len = 0;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// 比较两个以空字节结尾的字节串。
///
/// 返回值：
/// - `0`  ：两个字符串相等
/// - `< 0`：`s1` 按字典序小于 `s2`
/// - `> 0`：`s1` 按字典序大于 `s2`
///
/// # 安全性
/// `s1` 和 `s2` 都必须各指向合法的、以空字节结尾的字节串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    // TODO: 实现 strcmp
    let mut i = 0;
    loop {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };

        if a != b {
            return a as i32 - b as i32;
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
}

// ============================================================
// 测试（在 #[cfg(test)] 下可以使用 std）
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memcpy_basic() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 5) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memcpy_zero_len() {
        let src = [0xFFu8; 4];
        let mut dst = [0u8; 4];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 0) };
        assert_eq!(dst, [0u8; 4]);
    }

    #[test]
    fn test_memset_basic() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xAB, 8) };
        assert!(buf.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_memset_partial() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xFF, 4) };
        assert_eq!(&buf[..4], &[0xFF; 4]);
        assert_eq!(&buf[4..], &[0x00; 4]);
    }

    #[test]
    fn test_memmove_no_overlap() {
        let src = [1u8, 2, 3, 4];
        let mut dst = [0u8; 4];
        unsafe { my_memmove(dst.as_mut_ptr(), src.as_ptr(), 4) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memmove_overlap_forward() {
        // 把 buf[0..4] 复制到 buf[1..5]，整体右移 1 个字节
        let mut buf = [1u8, 2, 3, 4, 5];
        unsafe { my_memmove(buf.as_mut_ptr().add(1), buf.as_ptr(), 4) };
        assert_eq!(buf, [1, 1, 2, 3, 4]);
    }

    #[test]
    fn test_strlen_basic() {
        let s = b"hello\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 5);
    }

    #[test]
    fn test_strlen_empty() {
        let s = b"\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        assert_eq!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_less() {
        let a = b"abc\0";
        let b = b"abd\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strcmp_greater() {
        let a = b"abd\0";
        let b = b"abc\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } > 0);
    }
}

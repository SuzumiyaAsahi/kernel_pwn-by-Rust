use anyhow::{self, Context, Ok};
use colored::*;
use libc::{self, c_void};
// 当需要 使用到wait等待子进程完成时使用
// use nix::sys::wait::wait;
// 如果用到了Linux系统调用再用
// use nix::unistd::{nix, fork, write, ForkResult};
use regex::Regex;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::process::{self, Command};

fn main() -> anyhow::Result<()> {
    println!("{}", format!("{}", "Fuck you, world!".bold().purple()));
    Ok(())
}

// 当出现fork子进程的时候使用
// match unsafe { fork() } {
//     Result::Ok(ForkResult::Child) => {
//     }
//     Result::Ok(ForkResult::Parent { .. }) => {
//          unix::wait()?;
//     }
//     Result::Err(_) => {
//          process::exit(-1);
//     }
// }

// 当需要知道当前进程的uid时
// match libc::getuid() {
//     0 => {
//     }
//     _ => {
//          process::exit(-1);
//     }
// }

// 使用shell命令时
// 使用shell，但不需要输出的
//  std::process::Command::new("/bin/sh")
// .status()
// .expect("failed to execute process");

// 使用shell命令时
// 使用shell，同时要自己截获shell输出的
// let output = Command::new("sh")
//     .arg("-c")
//     .arg("dmesg | grep flag")
//     .output()
//     .expect("flag 执行失败");
// let output = String::from_utf8(output.stdout).expect("output转译失败");

// 获得当前系统页面大小
// let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };

// 生成指针数组
// let mut physmap_spray_arr: Vec<*mut c_void> = vec![std::ptr::null_mut(); 16000];

// mmap 使用方法
//         physmap_spray_arr[0] = libc::mmap(
//             std::ptr::null_mut(),
//             page_size as usize,
//             libc::PROT_READ | libc::PROT_WRITE,
//             libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
//             -1,
//             0,
//         );
//         construct_rop_chain(physmap_spray_arr[0] as *mut *const c_void, page_size);

//mmap 分配失败判断方法
// assert!(
//     physmap_spray_arr[index] != libc::MAP_FAILED as *mut _,
//     "oops for physmap spray!"
// );

// Rust 对于数组的理解还挺怪的，跟咱们想的不太一样，一维数组也要用二维指针来表示
// rop[i]在Rust用*rop.add(i)来表示
// fn construct_rop_chain(rop: *mut *const c_void, page_size: i64) {
//     let mut idx: usize = 0;
//     unsafe {
//         while idx < (page_size as usize / 8 - 0x30) {
//             *rop.add(idx) = add_rsp_0xa0_pop_rbx_pop_r12_pop_r13_pop_rbp_ret;
//             idx += 1;
//         }
//     }
// }

// memcpy的使用方法
// libc::memcpy(
//     physmap_spray_arr[index],
//     physmap_spray_arr[0],
//     page_size as libc::size_t,
// );

// anyhow Err 转化方法
// 突出的就是一个神兔二象性
// fn core_read(fd: i32, buf: &mut [u64]) -> anyhow::Result<()> {
//     match unsafe { libc::ioctl(fd, 0x6677889B, buf.as_mut_ptr()) } {
//         -1 => Err(anyhow::Error::new(io::Error::last_os_error()).context("core_read执行失败")),
//         _ => {
//             println!("{}", "core_read成功执行".red().bold());
//             Ok(())
//         }
//     }
// }

// sizeof 在 Rust 中的使用方法
// libc::write(
//     fd_num,
//     rop_chain.as_mut_ptr() as *const libc::c_void,
//     rop_chain.len() * std::mem::size_of::<u64>(),
// )

// 正则表达式截获数据  &caps[0]表示从整个字段中截获的第一组数据
// let output = String::from_utf8(output.stdout).expect("output转译失败");
// let re = Regex::new(r"([a-fA-F0-9]{16,})").expect("正则生成失败");
// let caps = re.captures(output.as_str()).unwrap();
// let flag_addr = &caps[0];

// 关于如何将字符串转换为u64类型数据的方法，其实一般情况下用 "123".parse::<u64>()更好
// 但这种方法只适用于处理十进制数字，当处理十六进制数时便不再适用了。
// let flag_addr = u64::from_str_radix(flag_addr, 16).unwrap();

// 不安全多线程的创造方法
// libc::pthread_create(
//     &mut thread,
//     std::ptr::null(),
//     competetion_thread,
//     std::ptr::null_mut(),
// );

// Rust 风味 C代码书写方法
// extern "C" fn competetion_thread(_: *mut libc::c_void) -> *mut libc::c_void {
//     while unsafe { status } {
//         for _ in 0..competetion_times {
//             unsafe { flag.flag_addr = std::mem::transmute(real_addr) }
//         }
//     }
//     std::ptr::null_mut()
// }

// 如何初始化为0
// let mut thread: libc::pthread_t = std::mem::zeroed();

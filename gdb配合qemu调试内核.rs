use std::process::Command;

fn main() {
    // 创建一个gdb子进程，并配置多个命令
    let mut gdb_process = Command::new("sudo")
        .arg("gdb")
        .arg("./bzImage")
        .arg("-q")
        .arg("-ex") // 为每个命令使用一个单独的 -ex
        .arg("add-symbol-file ./core/kgadget.ko 0xffffffffc0002000")
        .arg("-ex")
        .arg("target remote localhost:1234")
        .arg("-ex")
        .arg("b *(0x19A + 0xffffffffc0002000)") // 运行程序
        .arg("-ex")
        .arg("continue")
        .spawn()
        .expect("Failed to start gdb");

    // 等待 gdb 进程结束
    gdb_process.wait().expect("Failed to wait on gdb");
}

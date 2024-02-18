use std::process::{Command, Stdio};

#[derive(Debug, Default)]
pub struct RunnerOptions {
    pub gdb: bool,
    pub verbose: bool,
}

pub fn run(opts: &RunnerOptions) {
    let kernel_path = env!("KERNEL_PATH");
    let pre_kernel_path = env!("PRE_KERNEL_PATH");
    let mut cmd = Command::new("qemu-system-x86_64");
    let mut debugger = None;

    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.arg("-machine");
    cmd.arg("type=pc-i440fx-3.1");
    cmd.arg("-serial");
    cmd.arg("stdio");
    cmd.arg("-device");
    cmd.arg("isa-debug-exit");
    cmd.arg("-display");
    cmd.arg("gtk");
    cmd.arg("-kernel");
    cmd.arg(pre_kernel_path);
    cmd.arg("-initrd");
    cmd.arg(kernel_path);

    // debug options
    cmd.arg("-d");
    cmd.arg("guest_errors");
    cmd.arg("-d");
    cmd.arg("unimp");

    if opts.verbose {
        cmd.arg("-d");
        cmd.arg("int");
    }

    cmd.arg("-no-reboot");
    cmd.arg("-no-shutdown");

    if opts.gdb {
        cmd.arg("-gdb");
        cmd.arg("tcp::1234");
        cmd.arg("-S");

        let mut debug_cmd = Command::new("gdb");
        debug_cmd.stdin(Stdio::inherit());
        debug_cmd.stderr(Stdio::inherit());
        debug_cmd.stdout(Stdio::inherit());
        debug_cmd.arg("-ex");
        debug_cmd.arg("set confirm off");
        debug_cmd.arg("-ex");
        debug_cmd.arg("set disassembly-flavor intel");
        debug_cmd.arg("-ex");
        debug_cmd.arg("target remote localhost:1234");
        debug_cmd.arg("-ex");
        debug_cmd.arg(format!("add-symbol-file {pre_kernel_path}"));
        debug_cmd.arg("-ex");
        debug_cmd.arg("break _start");
        debug_cmd.arg("-ex");
        debug_cmd.arg("c");
        debug_cmd.arg("-ex");
        debug_cmd.arg("display/i $pc");

        debugger = Some(debug_cmd);
    }

    eprintln!("{cmd:?}");
    if let Some(debugger) = debugger.as_ref() {
        eprintln!("{debugger:?}");
    }

    let mut qemu_proc = cmd.spawn().unwrap();
    let debugger_proc = debugger.and_then(|mut cmd| cmd.spawn().ok());

    qemu_proc.wait().unwrap();
    debugger_proc.and_then(|mut proc| proc.wait().ok());
}

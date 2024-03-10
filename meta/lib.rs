use std::process::{Command, Stdio};

#[derive(Debug, Default)]
pub struct RunnerOptions {
    pub gdb: bool,
    pub verbose: bool,
    pub n_proc: Option<usize>,
}

pub fn run(opts: &RunnerOptions) {
    let kernel_path = env!("KERNEL_PATH");
    let pre_kernel_path = env!("PRE_KERNEL_PATH");
    let mut cmd = Command::new("qemu-system-x86_64");
    let mut debugger = None;

    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.args(["-machine", "type=pc-i440fx-3.1"]);
    cmd.args(["-serial", "stdio"]);
    cmd.args(["-device", "isa-debug-exit"]);
    cmd.args(["-display", "gtk"]);
    cmd.args(["-kernel", pre_kernel_path]);
    cmd.args(["-initrd", kernel_path]);

    // debug options
    cmd.args(["-d", "guest_errors"]);
    cmd.args(["-d", "unimp"]);

    if opts.verbose {
        cmd.args(["-d", "int"]);
    }

    let n_proc = opts.n_proc.unwrap_or(8);
    cmd.args(["-smp", &format!("{n_proc}")]);

    cmd.arg("-no-reboot");
    cmd.arg("-no-shutdown");

    if opts.gdb {
        cmd.args(["-gdb", "tcp::1234"]);
        cmd.arg("-S");

        let mut debug_cmd = Command::new("gdb");
        debug_cmd.stdin(Stdio::inherit());
        debug_cmd.stderr(Stdio::inherit());
        debug_cmd.stdout(Stdio::inherit());
        debug_cmd.args(["-ex", "set confirm off"]);
        debug_cmd.args(["-ex", "set disassembly-flavor intel"]);
        debug_cmd.args(["-ex", "target remote localhost:1234"]);
        debug_cmd.args(["-ex", &format!("add-symbol-file {pre_kernel_path}")]);
        debug_cmd.args(["-ex", "break _start"]);
        debug_cmd.args(["-ex", "c"]);
        debug_cmd.args(["-ex", "display/i $pc"]);

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

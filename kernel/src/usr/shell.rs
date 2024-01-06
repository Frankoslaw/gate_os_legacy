use crate::api::console::Style;
use crate::api::fs;
use crate::api::prompt::Prompt;
use crate::api::syscall;
use crate::{api, sys, usr};
use api::process::ExitCode;
use crate::sys::fs::FileType;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn prompt_string(success: bool) -> String {
    let csi_line1 = Style::color("LightCyan");
    let csi_line2 = Style::color("LightGray");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();

    let current_dir = sys::process::dir();

    let line1 = format!("{}{}{}", csi_line1, current_dir, csi_reset);
    let line2 = format!(
        "{}>{} ",
        if success { csi_line2 } else { csi_error },
        csi_reset
    );
    format!("{}\n{}", line1, line2)
}

pub fn exec(cmd: &str) -> Result<(), ExitCode> {
    let args: Vec<&str> = cmd.trim().split(' ').collect();

    if args.is_empty() {
        return Ok(());
    }

    let res = match args[0] {
        "" => Ok(()),
        "disk"     => usr::disk::main(&args),
        "elf"      => usr::elf::main(&args),
        "hello" => {
            println!("Hello world!");
            Ok(())
        },
        "hex"      => usr::hex::main(&args),
        "install"  => usr::install::main(&args),
        "list"     => usr::list::main(&args),
        "read"     => usr::read::main(&args),
        "quit"     => Err(ExitCode::ShellExit),
        "panic" => panic!("{}", args[1..].join(" ")),
        _          => {
            let mut path = fs::realpath(args[0]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            match syscall::info(&path).map(|info| info.kind()) {
                Some(FileType::Dir) => {
                    sys::process::set_dir(&path);
                    Ok(())
                }
                Some(FileType::File) => {
                    spawn(&path, &args)
                }
                _ => {
                    let path = format!("/bin/{}", args[0]);
                    spawn(&path, &args)
                }
            }
        }
    };

    res
}

fn spawn(path: &str, args: &[&str]) -> Result<(), ExitCode> {
    match api::process::spawn(path, args) {
        Err(ExitCode::ExecError) => {
            log::error!("Could not execute '{}'", args[0]);
            Err(ExitCode::ExecError)
        }
        Err(ExitCode::ReadError) => {
            log::error!("Could not read '{}'", args[0]);
            Err(ExitCode::ReadError)
        }
        Err(ExitCode::OpenError) => {
            log::error!("Could not open '{}'", args[0]);
            Err(ExitCode::OpenError)
        }
        res => res,
    }
}

fn repl() -> Result<(), ExitCode> {
    println!();

    let mut prompt = Prompt::new();

    let mut code = ExitCode::Success;
    while let Some(cmd) = prompt.input(&prompt_string(code == ExitCode::Success)) {
        code = match exec(&cmd) {
            Err(ExitCode::ShellExit) => break,
            Err(e) => e,
            Ok(()) => ExitCode::Success,
        };
        sys::console::drain();
        println!();
    }
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
    Ok(())
}

pub fn main() -> Result<(), ExitCode> {
    repl()
}

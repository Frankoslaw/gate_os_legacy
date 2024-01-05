use crate::api::console::Style;
use crate::api::prompt::Prompt;
use crate::{api, print, println, sys};
use api::process::ExitCode;

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

fn exec(cmd: &str) -> Result<(), ExitCode> {
    let args: Vec<&str> = cmd.trim().split(' ').collect();

    if args.is_empty() {
        return Ok(());
    }

    let res = match args[0] {
        "" => Ok(()),
        "panic" => panic!("{}", args[1..].join(" ")),
        "hello" => {
            println!("Hello world!");
            Ok(())
        }
        _ => {
            println!("Command not found");
            Ok(())
        }
    };

    res
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

//! Example: spawn a command with shell expansion (blocking).
//!
//! Run with:
//!
//! ```sh
//! cargo run --example std_expand --features expand
//! ```

use io_hook::exec::*;
use io_notify::{notification::Notification, runtimes::std::handle as n};
use io_process::{command::Command, runtimes::std::handle as p};

fn main() {
    env_logger::init();

    let notif = Notification {
        summary: "io-hook".into(),
        body: "Notification from io-notify".into(),
    };

    let mut arg = None;
    let mut coroutine = HookExec::new(notif);

    loop {
        match coroutine.resume(arg.take()) {
            HookExecResult::Ok => break,
            HookExecResult::NotifyIo { input } => arg = Some(n(input).unwrap().into()),
            HookExecResult::ProcessIo { input } => arg = Some(p(input).unwrap().into()),
            HookExecResult::Err { err } => panic!("{err}"),
        }
    }

    let mut cmd = Command::new("notify-send");
    cmd.arg("io-hook").arg("Notification from io-process");

    let mut arg = None;
    let mut coroutine = HookExec::new(cmd);

    loop {
        match coroutine.resume(arg.take()) {
            HookExecResult::Ok => break,
            HookExecResult::NotifyIo { input } => arg = Some(n(input).unwrap().into()),
            HookExecResult::ProcessIo { input } => arg = Some(p(input).unwrap().into()),
            HookExecResult::Err { err } => panic!("{err}"),
        }
    }
}

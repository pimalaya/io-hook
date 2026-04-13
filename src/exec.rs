use core::mem;

#[cfg(feature = "notify")]
use io_notify::{coroutines::send::*, io::*};
#[cfg(feature = "command")]
use io_process::{coroutines::spawn::*, io::*};
use log::trace;
use thiserror::Error;

use crate::hook::Hook;

/// Error emitted by the [`Notify`] coroutine.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum HookExecError {
    #[error("Invalid Hook arg {arg:?} for state {state:?}")]
    Invalid {
        arg: Option<HookExecArg>,
        state: HookExecState,
    },

    #[cfg(feature = "command")]
    #[error(transparent)]
    Process(#[from] ProcessSpawnError),
    #[cfg(feature = "notify")]
    #[error(transparent)]
    Notify(#[from] NotifySendError),
}

/// Result emitted on each step of the [`Hook`] coroutine.
#[derive(Debug)]
pub enum HookExecResult {
    /// The coroutine has successfully terminated its progression.
    Ok,
    /// A process I/O needs to be performed to make the coroutine
    /// progress.
    #[cfg(feature = "notify")]
    NotifyIo { input: NotifyInput },
    #[cfg(feature = "command")]
    ProcessIo { input: ProcessInput },
    /// The coroutine encountered an unrecoverable error.
    Err { err: HookExecError },
}

#[derive(Debug, Default)]
pub enum HookExecState {
    #[cfg(feature = "command")]
    ProcessSpawn(ProcessSpawn),
    #[cfg(feature = "notify")]
    NotifySend(NotifySend),
    #[default]
    Invalid,
}

/// I/O-free coroutine for Hooking a process and waiting for its exit
/// status.
///
/// Use this when you only care about whether the process succeeded or
/// failed. To also capture stdout and stderr, see [`HookOut`].
///
/// [`HookOut`]: super::Hook_out::HookOut
#[derive(Debug)]
pub struct HookExec {
    state: HookExecState,
}

#[derive(Debug)]
pub enum HookExecArg {
    None,
    #[cfg(feature = "command")]
    Command(ProcessOutput),
    #[cfg(feature = "notify")]
    Notif(NotifyOutput),
}

#[cfg(feature = "command")]
impl From<HookExecArg> for Option<ProcessOutput> {
    fn from(arg: HookExecArg) -> Self {
        match arg {
            HookExecArg::Command(arg) => Some(arg),
            _ => None,
        }
    }
}

#[cfg(feature = "command")]
impl From<ProcessOutput> for HookExecArg {
    fn from(arg: ProcessOutput) -> Self {
        Self::Command(arg)
    }
}

#[cfg(feature = "notify")]
impl From<HookExecArg> for Option<NotifyOutput> {
    fn from(arg: HookExecArg) -> Self {
        match arg {
            HookExecArg::Notif(arg) => Some(arg),
            _ => None,
        }
    }
}

#[cfg(feature = "notify")]
impl From<NotifyOutput> for HookExecArg {
    fn from(arg: NotifyOutput) -> Self {
        Self::Notif(arg)
    }
}

impl HookExec {
    /// Creates a new coroutine that will Hook the given command.
    pub fn new(hook: impl Into<Hook>) -> Self {
        trace!("prepare hook");

        let state = match hook.into() {
            #[cfg(feature = "command")]
            Hook::Command(cmd) => HookExecState::ProcessSpawn(ProcessSpawn::new(cmd)),
            #[cfg(feature = "notify")]
            Hook::Notify(notif) => HookExecState::NotifySend(NotifySend::new(notif)),
        };

        Self { state }
    }

    /// Makes the Hook progress.
    pub fn resume(&mut self, arg: impl Into<Option<HookExecArg>>) -> HookExecResult {
        match (&mut self.state, arg.into()) {
            #[cfg(feature = "command")]
            (HookExecState::ProcessSpawn(c), arg) => match c.resume(arg.and_then(Into::into)) {
                ProcessSpawnResult::Ok { .. } => HookExecResult::Ok,
                ProcessSpawnResult::Io { input } => {
                    let input = input.into();
                    HookExecResult::ProcessIo { input }
                }
                ProcessSpawnResult::Err { err } => {
                    let err = err.into();
                    HookExecResult::Err { err }
                }
            },
            #[cfg(feature = "notify")]
            (HookExecState::NotifySend(c), arg) => match c.resume(arg.and_then(Into::into)) {
                NotifySendResult::Ok => HookExecResult::Ok,
                NotifySendResult::Io { input } => {
                    let input = input.into();
                    HookExecResult::NotifyIo { input }
                }
                NotifySendResult::Err { err } => {
                    let err = err.into();
                    HookExecResult::Err { err }
                }
            },
            (state, arg) => {
                let state = mem::take(state);
                let err = HookExecError::Invalid { arg, state };
                HookExecResult::Err { err }
            }
        }
    }
}

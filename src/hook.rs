#[cfg(feature = "notify")]
use io_notify::notification::Notification;
#[cfg(feature = "command")]
use io_process::command::Command;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum Hook {
    #[cfg(feature = "command")]
    Command(Command),
    #[cfg(feature = "notify")]
    Notify(Notification),
}

#[cfg(feature = "command")]
impl From<Command> for Hook {
    fn from(cmd: Command) -> Self {
        Self::Command(cmd)
    }
}

#[cfg(feature = "notify")]
impl From<Notification> for Hook {
    fn from(notif: Notification) -> Self {
        Self::Notify(notif)
    }
}

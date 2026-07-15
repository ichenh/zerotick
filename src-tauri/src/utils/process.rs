//! Shared configuration for background child processes.

use std::process::Command;

pub trait CommandExt {
    /// Prevent Windows from allocating a visible console for background commands.
    fn hide_window(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn hide_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as WindowsCommandExt;

            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            self.creation_flags(CREATE_NO_WINDOW);
        }
        self
    }
}

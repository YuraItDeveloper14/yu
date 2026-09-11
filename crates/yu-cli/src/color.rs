/// ANSI colours for error messages; off for pipes, `NO_COLOR` and `--no-color`.
#[derive(Clone, Copy)]
pub struct Paint {
    on: bool,
}

impl Paint {
    pub fn new(wanted: bool) -> Self {
        Paint {
            on: wanted && enable_vt(),
        }
    }

    fn wrap(&self, code: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }

    /// Red header, plain source line, yellow carets, cyan hint.
    pub fn diagnostic(&self, rendered: &str) -> String {
        rendered
            .lines()
            .enumerate()
            .map(|(i, line)| match i {
                0 => self.wrap("1;31", line),
                1 => line.to_string(),
                2 => self.wrap("33", line),
                _ => self.wrap("36", line),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Windows consoles need virtual-terminal mode switched on before ANSI codes work.
#[cfg(windows)]
fn enable_vt() -> bool {
    type Handle = *mut core::ffi::c_void;
    extern "system" {
        fn GetStdHandle(which: u32) -> Handle;
        fn GetConsoleMode(handle: Handle, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: Handle, mode: u32) -> i32;
    }
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    // SAFETY: plain Win32 calls on this process's own stderr handle.
    unsafe {
        let handle = GetStdHandle(STD_ERROR_HANDLE);
        let mut mode = 0;
        GetConsoleMode(handle, &mut mode) != 0
            && SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

#[cfg(not(windows))]
fn enable_vt() -> bool {
    true
}

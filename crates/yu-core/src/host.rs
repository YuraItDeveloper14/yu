/// Everything a program does to the outside world goes through the host:
/// the terminal in `yu`, the page in the playground, a buffer in tests.
pub trait Host {
    /// One line printed by `скажи`.
    fn print(&mut self, line: &str);
    /// A line the user typed after `prompt`; `None` when there is no more input.
    fn ask(&mut self, prompt: &str) -> Option<String>;
}

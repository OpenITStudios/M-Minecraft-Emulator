#![forbid(unsafe_code)]

// Platform-neutral Minecraft session contracts and the bounded subprocess runner.
// VMware/Guest Agent/App Sandbox lifecycle code is retired from this crate.
pub mod independent_engine;
mod inspection_budget;
mod process;
pub use inspection_budget::InspectionBudget;

// The desktop uses this bounded process runner for reviewed on-demand helpers.
#[doc(hidden)]
pub fn bounded_command_output<I, S>(
    program: &std::path::Path,
    args: I,
    timeout: std::time::Duration,
) -> std::io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    process::command_output_with_timeout(program, args, timeout)
}

// Pass large fixed scripts via a bounded stdin pipe rather than the Windows
// command line. The shared process runner keeps the same timeout and
// output-size limits, and manages only its direct child.
#[doc(hidden)]
pub fn bounded_command_output_with_stdin<I, S>(
    program: &std::path::Path,
    args: I,
    timeout: std::time::Duration,
    input: &[u8],
) -> std::io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    process::command_output_with_stdin(program, args, timeout, input)
}

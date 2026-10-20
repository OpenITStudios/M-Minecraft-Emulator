#![windows_subsystem = "windows"]
#[path = "../process.rs"]
mod process;
fn main() {
    let output = std::env::args_os().nth(1).unwrap();
    let powershell=std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let script=r#"Add-Type 'using System;using System.Runtime.InteropServices;public static class ConsoleProbe{[DllImport("kernel32.dll")]public static extern IntPtr GetConsoleWindow();}';[ConsoleProbe]::GetConsoleWindow().ToInt64()"#;
    let result=process::command_output_with_timeout(&powershell,["-NoProfile","-NonInteractive","-Command",script],std::time::Duration::from_secs(10)).unwrap();
    std::fs::write(output,result).unwrap();
}

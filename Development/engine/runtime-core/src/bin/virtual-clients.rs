#![forbid(unsafe_code)]

use m_bedrock_virtual_clients_core::execute_public_command;

fn print_help() {
    println!("M-Bedrock Virtual Clients — native-first feasibility");
    println!("  product-status");
    println!("  doctor / status / snapshot / diagnostics / history (legacy read-only)");
    println!("  policy / base-preflight / resources <1-3> (legacy reference only)");
    println!("  support-bundle / check-update (read-only)");
    println!("  app-sandbox-pilot inspect (explicit read-only diagnostic)");
    println!("Legacy VM start, stop, provision, snapshot mutation, reset and");
    println!("App Sandbox pilot power operations are paused; no account data removed.");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());
    if command == "help" || command == "--help" || command == "-h" {
        print_help();
        return;
    }

    let args = args.collect::<Vec<_>>();
    let result = execute_public_command(&command, &args);
    if result.success {
        println!("{}", result.json);
    } else {
        eprintln!("{}", result.json);
        std::process::exit(1);
    }
}

use inquire::{Confirm, Select, Text};
use rand::{distributions::Alphanumeric, Rng};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("--- Shoko Key Generator (skgenkey) ---");
    println!("This utility will help you generate a high-security 32-byte key for SHOKO_KEY.\n");

    let key_name = Text::new("What is the name/alias for this key?")
        .with_default("main-vault-key")
        .prompt()
        .unwrap_or_else(|_| "shoko-key".to_string());

    let options = vec!["Standard (Alphanumeric)", "High (Full ASCII Symbols)"];
    let complexity = Select::new("Select key complexity:", options).prompt().unwrap();

    let key: String = if complexity == "Standard (Alphanumeric)" {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    } else {
        (0..32)
            .map(|_| {
                let c = rand::thread_rng().gen_range(33..126) as u8;
                c as char
            })
            .collect()
    };

    let separator = "=".repeat(40);
    println!("\n{}", separator);
    println!("GENERATED KEY [Name: {}]", key_name);
    println!("{}", key);
    println!("{}", separator);
    
    println!("\nCRITICAL: Copy this key now. It will be wiped from the screen shortly.");
    println!("To use it, add this to your .bashrc or .zshrc:");
    println!("export SHOKO_KEY=\"{}\"", key);

    let _ = Confirm::new("Have you copied the key and saved it in a secure place?")
        .with_default(false)
        .prompt();

    print!("\x1B[12A\x1B[J");
    io::stdout().flush()?;

    println!("SUCCESS: Key wiped from terminal display.");
    println!("Note: If you haven't exported it yet, run:");
    println!("export SHOKO_KEY=\"your_copied_key\"");

    Ok(())
}

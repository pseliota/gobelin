use std::{collections::HashMap, process::ExitCode};

use gobelin::alpaca::trading::AlpacaTradingClient;

fn credential(names: &[&str], file_values: &HashMap<String, String>) -> Result<String, String> {
    let value = names
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|v| !v.trim().is_empty()))
        .or_else(|| {
            names.iter().find_map(|name| {
                file_values
                    .get(*name)
                    .filter(|v| !v.trim().is_empty())
                    .cloned()
            })
        })
        .ok_or_else(|| {
            format!(
                "Missing credential: set {} in .env or the environment",
                names[0]
            )
        })?;

    Ok(value)
}

fn run() -> Result<(), String> {
    // Parse locally rather than changing the process environment or logging secrets.
    let file_values: HashMap<String, String> = match dotenvy::from_path_iter(".env") {
        Ok(entries) => entries
            .collect::<Result<_, _>>()
            .map_err(|_| "Could not parse .env; check its KEY=value syntax".to_owned())?,
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            HashMap::new()
        }
        Err(_) => return Err("Could not read .env".to_owned()),
    };

    let key = credential(
        &["ALPACA_KEY", "ALPACA_API_KEY", "APCA_API_KEY_ID"],
        &file_values,
    )?;
    let secret = credential(
        &[
            "ALPACA_SECRET",
            "ALPACA_SECRET_KEY",
            "ALPACA_API_SECRET",
            "APCA_API_SECRET_KEY",
        ],
        &file_values,
    )?;

    let client = AlpacaTradingClient::new(&key, &secret).map_err(|error| error.to_string())?;
    let account = client.get_account().map_err(|error| error.to_string())?;

    println!("Alpaca paper account (read-only request)");
    println!("Status: {}", account.status);
    println!(
        "Currency: {}",
        account.currency.as_deref().unwrap_or("unavailable")
    );
    println!("Cash: {}", account.cash.as_deref().unwrap_or("unavailable"));
    println!(
        "Equity: {}",
        account.equity.as_deref().unwrap_or("unavailable")
    );
    println!(
        "Buying power: {}",
        account.buying_power.as_deref().unwrap_or("unavailable")
    );
    println!(
        "Trading blocked: {}",
        match account.trading_blocked {
            Some(true) => "true",
            Some(false) => "false",
            None => "unavailable",
        }
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

use std::process::ExitCode;

use gobelin::{alpaca::trading::AlpacaTradingClient, config::Credentials};

fn run() -> Result<(), String> {
    let credentials = Credentials::load().map_err(|error| error.to_string())?;
    let key = credentials
        .alpaca_key()
        .map_err(|error| error.to_string())?;
    let secret = credentials
        .alpaca_secret()
        .map_err(|error| error.to_string())?;

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

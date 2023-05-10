mod client;
mod limiter;

use std::time::Duration;

use client::Client;

use spacedust::models::ship_frame::Symbol;
use tokio::time::interval;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let ships = client.get_my_ships().await;

    let drones: Vec<_> = ships
        .iter()
        .filter(|s| s.frame.symbol == Symbol::Drone)
        .collect();

    for d in &drones {
        dbg!((&d.symbol, &d.nav));
    }

    for d in &drones {
        let ship_symbol = d.symbol.to_owned();
        let client = Client::new();

        tokio::spawn(async move {
            loop {
                println!("[{ship_symbol}] docking");
                client.dock_ship(&ship_symbol).await;

                println!("[{ship_symbol}] emptying");
                client.sell_all(&ship_symbol).await;

                println!("[{ship_symbol}] orbit");
                client.orbit_ship(&ship_symbol).await;

                println!("[{ship_symbol}] extract");
                client.extract_till_full(&ship_symbol).await;
            }
        });

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let mut stream = interval(Duration::from_secs(60));
    loop {
        stream.tick().await;

        let m = client.get_my_agent().await;
        println!("[MAIN] credits={}", m.credits)
    }
}

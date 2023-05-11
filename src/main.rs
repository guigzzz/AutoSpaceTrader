mod client;
mod configuration;
mod limiter;
mod manager;

use std::time::Duration;

use client::Client;

use manager::Manager;
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

    tokio::spawn(async {
        let manager = Manager::new("BUYER").await;
        println!("[BUYER] Init manager done");

        let client = Client::new();
        let current_system = client.get_ship("MXZ-1").await.nav.system_symbol;

        let mut stream = interval(Duration::from_secs(600));
        loop {
            stream.tick().await;

            println!("[BUYER] Checking for funds");

            let ships = client.get_my_ships().await;
            if ships.len() > 10 {
                println!("[BUYER] Already have 10 ships, quitting...");
                return;
            }

            let m = client.get_my_agent().await;
            if m.credits > 100_000 {
                println!("[BUYER] Enough credits for ship, attempting to buy");
                manager
                    .buy_ship_and_send_mining(current_system.as_str())
                    .await
            } else {
                println!("[BUYER] Not enough funds");
            }
        }
    });

    let mut stream = interval(Duration::from_secs(60));
    loop {
        stream.tick().await;

        let m = client.get_my_agent().await;
        println!("[MAIN] credits={}", m.credits)
    }
}

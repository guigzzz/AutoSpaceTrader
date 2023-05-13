mod client;
mod configuration;
mod limiter;
mod manager;

use std::time::Duration;

use client::Client;

use manager::ManagerFactory;
use spacedust::models::ship_frame::Symbol;
use tokio::time::interval;

#[tokio::main(worker_threads = 1)]
async fn main() {
    let client = Client::new("MAIN".into());

    let ships = client.get_my_ships().await;

    dbg!(ships
        .iter()
        .map(|s| s.symbol.to_owned())
        .collect::<Vec<_>>());

    let drones: Vec<_> = ships
        .iter()
        .filter(|s| s.frame.symbol == Symbol::Drone || s.frame.symbol == Symbol::Miner)
        .collect();

    let factory = ManagerFactory::new().await;

    for d in &drones {
        let ship_symbol = d.symbol.to_owned();
        let manager = factory.get(&ship_symbol);

        tokio::spawn(async move {
            loop {
                manager.mine_loop(ship_symbol.as_str()).await;
            }
        });

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let manager = factory.get("BUYER");

    tokio::spawn(async move {
        println!("[BUYER] Init manager done");

        let client = Client::new("BUYER".into());
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
            if m.credits > 165_000 {
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
    }
}

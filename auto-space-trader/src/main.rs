mod ship;

use std::time::Duration;

use ship::Ship;

use spacedust::{
    apis::{
        agents_api::{self as agents},
        configuration::Configuration,
        fleet_api::{self as fleet},
    },
    models::ship_frame::Symbol,
};
use tokio::time::interval;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let configuration = Configuration {
        bearer_access_token: Some(env::var("TOKEN").unwrap()),
        ..Default::default()
    };

    let ships = fleet::get_my_ships(&configuration, None, None)
        .await
        .unwrap();

    let drones: Vec<_> = ships
        .data
        .iter()
        .filter(|s| s.frame.symbol == Symbol::Drone)
        .map(|s| {
            (
                s.registration.name.to_owned(),
                s.nav.to_owned(),
                s.cargo.to_owned(),
                s.fuel.to_owned(),
            )
        })
        .collect();

    for d in &drones {
        dbg!((&d.0, &d.1));
    }

    for d in &drones {
        let ship_name = d.0.to_owned();
        let configuration = configuration.to_owned();

        tokio::spawn(async move {
            loop {
                println!("[{ship_name}] docking");
                fleet::dock_ship(&configuration, &ship_name, 0.)
                    .await
                    .unwrap();

                println!("[{ship_name}] emptying");
                Ship::sell_all(&configuration, &ship_name).await;

                println!("[{ship_name}] orbit");
                fleet::orbit_ship(&configuration, &ship_name, 0)
                    .await
                    .unwrap();

                println!("[{ship_name}] extract");
                Ship::extract_till_full(&configuration, &ship_name).await;
            }
        });

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    let mut stream = interval(Duration::from_secs(60));
    loop {
        stream.tick().await;

        let m = agents::get_my_agent(&configuration).await.unwrap();
        println!("[MAIN] credits={}", m.data.credits)
    }
}

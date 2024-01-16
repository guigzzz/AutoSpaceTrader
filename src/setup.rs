use std::time::Duration;

use chrono::DateTime;
use log::info;
use spacedust::{
    apis::{
        configuration::Configuration, contracts_api, default_api::register, fleet_api, systems_api,
    },
    models::{
        NavigateShipRequest, PurchaseShipRequest, RegisterRequest, WaypointTraitSymbol,
        WaypointType,
    },
};

use crate::configuration::ConfigurationFactory;

pub struct Setup {}

impl Setup {
    pub async fn setup_account(username: &str) {
        info!("[SETUP] Registering");
        let agent = register(
            &Configuration::new(),
            Some(RegisterRequest::new(
                spacedust::models::FactionSymbol::Cosmic,
                username.to_string(),
            )),
        )
        .await
        .unwrap();

        let token = agent.data.token.as_str();
        info!("[SETUP] TOKEN={token}");

        let configuration = &ConfigurationFactory::get_config(agent.data.token.as_str());

        let contracts: Vec<_> = contracts_api::get_contracts(configuration, None, None)
            .await
            .unwrap()
            .data;
        info!("[SETUP] Found {} contracts", contracts.len());
        for contract in contracts.iter() {
            info!(
                "[SETUP] Contract {}: {}, {}, {}",
                contract.id,
                contract.terms.deadline,
                contract.terms.payment.on_accepted,
                contract.terms.payment.on_fulfilled
            )
        }

        for contract in contracts {
            info!("[SETUP] Accepting contract {}", contract.id);
            contracts_api::accept_contract(configuration, contract.id.as_str())
                .await
                .unwrap();
        }

        let ships = fleet_api::get_my_ships(configuration, None, None)
            .await
            .unwrap()
            .data;
        info!(
            "Ships: {}",
            ships
                .iter()
                .map(|s| s.symbol.to_owned())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let ship = fleet_api::get_my_ship(
            configuration,
            format!("{}-1", username.to_ascii_uppercase()).as_str(),
        )
        .await
        .unwrap()
        .data;

        let system = ship.nav.system_symbol.as_str();
        info!("[SETUP] Home system is {system}");

        let waypoints =
            systems_api::get_system_waypoints(configuration, system, None, None, None, None)
                .await
                .unwrap()
                .data;

        let shipyard = waypoints.iter().find(|w| {
            w.traits
                .iter()
                .any(|t| t.symbol == WaypointTraitSymbol::Shipyard)
        });

        let shipyard = match shipyard {
            None => panic!(
                "Failed to find shipyard in system {system}. Waypoints were: {}",
                waypoints
                    .iter()
                    .map(|w| w.symbol.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Some(s) => s,
        };

        info!("[SETUP] Found shipyard: {}", shipyard.symbol);

        loop {
            info!("[SETUP] Buying ship");

            let purchase = fleet_api::purchase_ship(
                configuration,
                Some(PurchaseShipRequest::new(
                    spacedust::models::ShipType::MiningDrone,
                    shipyard.symbol.to_owned(),
                )),
            )
            .await;

            match purchase {
                Result::Ok(_) => continue,
                Result::Err(e) => {
                    info!("[SETUP] Failed to buy ship, assuming out of money");
                    dbg!(e);
                    break;
                }
            }
        }

        let asteroid_field = waypoints
            .iter()
            .find(|w| w.r#type == WaypointType::AsteroidField)
            .unwrap();

        info!("[SETUP] Found asteroid field: {}", asteroid_field.symbol);

        let ships = fleet_api::get_my_ships(configuration, None, None)
            .await
            .unwrap()
            .data;

        if ships.len() == 1 {
            panic!("We only have one ship ? Did we fail to buy any ?")
        }

        let mut wait_seconds = None;
        for s in ships {
            info!(
                "[SETUP] Navigating {} to {}",
                s.symbol, asteroid_field.symbol
            );
            let nav = fleet_api::navigate_ship(
                configuration,
                s.symbol.as_str(),
                Some(NavigateShipRequest::new(asteroid_field.symbol.to_string())),
            )
            .await
            .unwrap()
            .data;

            let route = nav.nav.route;
            let departure = DateTime::parse_from_rfc3339(route.departure_time.as_str()).unwrap();
            let arrival = DateTime::parse_from_rfc3339(route.arrival.as_str()).unwrap();

            let eta = (arrival - departure).num_seconds() as u64;

            wait_seconds = wait_seconds.map_or(Some(eta), |v| Some(u64::max(v, eta)));
        }

        let wait = wait_seconds.unwrap();
        info!("[SETUP] Ships travelling to asteroid field, waiting {wait} seconds");
        tokio::time::sleep(Duration::from_secs(wait)).await;

        info!("[SETUP] Ready to go!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};

    #[tokio::test]
    async fn test_setup() {
        init_logging();

        let user = random_string();

        info!("user: {}", user);

        Setup::setup_account(user.as_str()).await
    }

    fn random_string() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    fn init_logging() {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {}] {}",
                    humantime::format_rfc3339(std::time::SystemTime::now()),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Info)
            .chain(std::io::stdout())
            .apply()
            .unwrap();
    }
}

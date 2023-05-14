use std::collections::HashMap;

use spacedust::models::{
    self, waypoint_trait::Symbol, ShipType, System, SystemWaypoint, Waypoint, WaypointType,
};

use crate::client::Client;

use log::info;

#[derive(Clone)]
pub struct ManagerFactory {
    systems: HashMap<String, System>,
}

impl ManagerFactory {
    pub async fn new() -> Self {
        let client = Client::new("MANAGER_FACTORY".to_owned());
        let systems: HashMap<String, System> = client
            .get_systems_all()
            .await
            .iter()
            .map(|s| (s.symbol.to_owned(), s.to_owned()))
            .collect();

        info!("[MANAGER_FACTORY] Loaded systems config");

        Self { systems }
    }

    pub fn get(&self, log_context: &str) -> Manager {
        Manager::new(log_context, self.systems.clone())
    }
}

#[derive(Clone)]
pub struct Manager {
    systems: HashMap<String, System>,
    log_context: String,
    client: Client,
}

impl Manager {
    fn new(log_context: &str, systems: HashMap<String, System>) -> Self {
        let client = Client::new(log_context.to_owned());
        Self {
            systems,
            log_context: log_context.to_owned(),
            client,
        }
    }

    pub async fn buy_ship_and_send_mining(&self, factory: &ManagerFactory, system_symbol: &str) {
        let ship = self
            .purchase_ship(system_symbol, spacedust::models::ShipType::OreHound)
            .await;
        info!(
            "[{}] Manager - Purchased ship: {} - {:?}",
            self.log_context, ship.symbol, ship
        );

        let asteroid_waypoint = self
            .find_waypoint_for_type(system_symbol, WaypointType::AsteroidField)
            .await
            .unwrap();
        info!(
            "[{}] Manager - Found AsteroidField waypoint: {}",
            self.log_context, asteroid_waypoint.symbol
        );
        self.client
            .navigate(ship.symbol.as_str(), asteroid_waypoint.symbol.as_str())
            .await;

        let manager = factory.get(ship.symbol.as_str());
        tokio::spawn(async move {
            loop {
                manager.mine_loop(ship.symbol.as_str()).await;
            }
        });
    }

    pub async fn mine_loop(&self, ship_symbol: &str) {
        let context = &self.log_context;
        info!("[{context}] docking");
        self.client.dock_ship(ship_symbol).await;

        info!("[{context}] emptying");
        self.client.sell_all(ship_symbol).await;

        info!("[{context}] orbit");
        self.client.orbit_ship(ship_symbol).await;

        info!("[{context}] extract");
        self.client.extract_till_full(ship_symbol).await;
    }

    pub async fn find_waypoint_for_type(
        &self,
        system_name: &str,
        waypoint_type: WaypointType,
    ) -> Option<SystemWaypoint> {
        let system = self.systems.get(system_name).unwrap();

        system
            .waypoints
            .iter()
            .cloned()
            .find(|w| w.r#type == waypoint_type)
    }

    pub async fn find_waypoint_for_trait(
        &self,
        system_name: &str,
        waypoint_trait: Symbol,
    ) -> Option<Waypoint> {
        let waypoints = self.client.get_system_waypoints(system_name).await;

        waypoints
            .iter()
            .cloned()
            .find(|w| w.traits.iter().any(|t| t.symbol == waypoint_trait))
    }

    pub async fn purchase_ship(&self, system_name: &str, ship_type: ShipType) -> Box<models::Ship> {
        let shipyard = self
            .find_waypoint_for_trait(system_name, Symbol::Shipyard)
            .await
            .unwrap();

        self.client
            .purchase_ship(ship_type, shipyard.symbol.as_str())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use spacedust::models::{
        ship_module::Symbol,
        ship_mount::{self},
    };

    #[tokio::test]
    async fn test() {
        let client = Client::new("bla".into());

        let ships = client.get_my_ships().await;

        println!(
            "{}",
            ships
                .iter()
                .map(|s| format!(
                    "{}-{}",
                    s.symbol,
                    s.mounts
                        .iter()
                        .find(|s| s.symbol == ship_mount::Symbol::MiningLaserI
                            || s.symbol == ship_mount::Symbol::MiningLaserIi)
                        .unwrap()
                        .name
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    // #[tokio::test]
    // async fn test_manager() {
    //     init_logging();

    //     let factory = ManagerFactory::new().await;

    //     let manager = factory.get("TEST");

    //     let eligible_waypoints: Vec<_> = manager
    //         .systems
    //         .iter()
    //         .filter_map(|(_, system)| {
    //             system
    //                 .waypoints
    //                 .iter()
    //                 .find(|w| w.r#type == WaypointType::AsteroidField)
    //                 .map(|w| (system.symbol.to_owned(), w))
    //         })
    //         .collect();

    //     info!("Found {} asteroid fields", eligible_waypoints.len());

    //     let w: Vec<_> = futures::stream::iter(eligible_waypoints)
    //         .map()
    //         .filter(|(system_symbol, waypoint)| async {
    //             let client = Client::new("TEST".into());

    //             let waypoints = client.get_system_waypoints(system_symbol).await;

    //             let waypoint = waypoints
    //                 .iter()
    //                 .find(|w| w.symbol == waypoint.symbol)
    //                 .unwrap();

    //             waypoint
    //                 .traits
    //                 .iter()
    //                 .any(|w| w.symbol == Symbol::Marketplace)
    //         })
    //         .collect()
    //         .await;
    // }

    // fn init_logging() {
    //     fern::Dispatch::new()
    //         .format(|out, message, record| {
    //             out.finish(format_args!(
    //                 "[{} {}] {}",
    //                 humantime::format_rfc3339(std::time::SystemTime::now()),
    //                 record.level(),
    //                 message
    //             ))
    //         })
    //         .level(log::LevelFilter::Info)
    //         .chain(std::io::stdout())
    //         .apply()
    //         .unwrap();
    // }
}

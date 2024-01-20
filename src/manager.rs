use spacedust::models::{self, ShipType, Waypoint, WaypointTraitSymbol, WaypointType};

use crate::client::Client;

use log::info;

#[derive(Clone)]
pub struct ManagerFactory {}

impl ManagerFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, log_context: &str) -> Manager {
        Manager::new(log_context)
    }
}

#[derive(Clone)]
pub struct Manager {
    log_context: String,
    client: Client,
}

impl Manager {
    fn new(log_context: &str) -> Self {
        let client = Client::new(log_context.to_owned());
        Self {
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
    ) -> Option<Waypoint> {
        let waypoints = self.client.get_system_waypoints(system_name).await;

        waypoints
            .iter()
            .find(|&w| w.r#type == waypoint_type)
            .cloned()
    }

    pub async fn find_waypoint_for_trait(
        &self,
        system_name: &str,
        waypoint_trait: WaypointTraitSymbol,
    ) -> Option<Waypoint> {
        let waypoints = self.client.get_system_waypoints(system_name).await;

        waypoints
            .iter()
            .find(|&w| w.traits.iter().any(|t| t.symbol == waypoint_trait))
            .cloned()
    }

    pub async fn purchase_ship(&self, system_name: &str, ship_type: ShipType) -> Box<models::Ship> {
        let shipyard = self
            .find_waypoint_for_trait(system_name, WaypointTraitSymbol::Shipyard)
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
    use spacedust::models::ship_mount::{self};

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

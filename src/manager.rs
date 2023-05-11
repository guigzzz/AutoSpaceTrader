use std::collections::HashMap;

use spacedust::{
    apis::{configuration::Configuration, fleet_api, systems_api},
    models::{
        self, waypoint_trait::Symbol, PurchaseShipRequest, ShipType, System, SystemWaypoint,
        Waypoint, WaypointType,
    },
};

use crate::{client::Client, configuration::CONFIGURATION};

pub struct Manager {
    configuration: &'static Configuration,
    systems: HashMap<String, System>,
    log_context: String,
    client: Client,
}

impl Manager {
    pub async fn new(log_context: &str) -> Self {
        let configuration = &CONFIGURATION;
        let systems: HashMap<String, System> = systems_api::get_systems_all(configuration)
            .await
            .unwrap()
            .iter()
            .map(|s| (s.symbol.to_owned(), s.to_owned()))
            .collect();

        println!(
            "[{log_context}] Manager Init - Found {} systems",
            systems.len()
        );

        Self {
            configuration,
            systems,
            log_context: log_context.to_owned(),
            client: Client::new(log_context.to_owned()),
        }
    }

    pub async fn get_system_waypoints(&self, system_name: &str) -> Vec<models::Waypoint> {
        systems_api::get_system_waypoints(self.configuration, system_name, None, None)
            .await
            .unwrap()
            .data
    }

    pub async fn buy_ship_and_send_mining(&self, system_symbol: &str) {
        let ship = self
            .purchase_ship(system_symbol, spacedust::models::ShipType::MiningDrone)
            .await;
        println!(
            "[{}] Manager - Purchased ship: {}",
            self.log_context, ship.symbol
        );

        let asteroid_waypoint = self
            .find_waypoint_for_type(system_symbol, WaypointType::AsteroidField)
            .await
            .unwrap();
        println!(
            "[{}] Manager - Found AsteroidField waypoint: {}",
            self.log_context, asteroid_waypoint.symbol
        );
        self.client
            .navigate(ship.symbol.as_str(), asteroid_waypoint.symbol.as_str())
            .await;
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
        let waypoints = self.get_system_waypoints(system_name).await;

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

        fleet_api::purchase_ship(
            self.configuration,
            Some(PurchaseShipRequest::new(ship_type, shipyard.symbol)),
        )
        .await
        .unwrap()
        .data
        .ship
    }
}

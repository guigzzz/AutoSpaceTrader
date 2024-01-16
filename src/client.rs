use std::{fmt::Debug, time::Duration};

use chrono::DateTime;
use spacedust::{
    apis::{
        agents_api,
        configuration::Configuration,
        fleet_api::{self as fleet},
        systems_api, Error,
    },
    models::{
        self, Agent, ExtractResourcesRequest, NavigateShipRequest, PurchaseShipRequest,
        SellCargoRequest, Ship, ShipType, System, TradeSymbol,
    },
};

use log::info;

use serde::{de::DeserializeOwned, Deserialize};

use crate::configuration::CONFIGURATION;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ExtractResourceError {
    Cooldown { cooldown: CoolDownErrorInner },
    Cargo(CargoErrorInner),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum SellCargoError {
    NotFoundError(NotFoundErrorInner),
    NotSellableError(NotSellableErrorInner),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotFoundErrorInner {
    ship_symbol: String,
    trade_symbol: String,
    cargo_units: u64,
    units_to_remove: u64,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotSellableErrorInner {
    trade_symbol: String,
    waypoint_symbol: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericErrorInner<T> {
    code: u16,
    data: T,
    message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericError<T> {
    error: GenericErrorInner<T>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoolDownErrorInner {
    expiration: String,
    remaining_seconds: u64,
    ship_symbol: String,
    total_seconds: u64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CargoErrorInner {
    ship_symbol: String,
}

#[derive(Clone)]
pub struct Client {
    configuration: &'static Configuration,
    log_context: String,
}

impl<T: Debug, U: DeserializeOwned> From<Error<T>> for GenericError<U> {
    fn from(value: Error<T>) -> Self {
        match value {
            Error::ResponseError(e) => match serde_json::from_str(&e.content) {
                Result::Ok(v) => v,
                Result::Err(ser_err) => {
                    dbg!(e);
                    dbg!(ser_err);
                    panic!()
                }
            },
            _ => panic!("{}", dbg!(value)),
        }
    }
}

impl Client {
    pub fn new(log_context: String) -> Self {
        Self {
            configuration: &CONFIGURATION,
            log_context,
        }
    }

    pub async fn get_systems_all(&self) -> Vec<System> {
        systems_api::get_systems(self.configuration, None, None)
            .await
            .unwrap()
            .data
    }

    pub async fn get_my_agent(&self) -> Box<Agent> {
        agents_api::get_my_agent(self.configuration)
            .await
            .unwrap()
            .data
    }

    pub async fn purchase_ship(
        &self,
        ship_type: ShipType,
        waypoint_symbol: &str,
    ) -> Box<models::Ship> {
        fleet::purchase_ship(
            self.configuration,
            Some(PurchaseShipRequest::new(
                ship_type,
                waypoint_symbol.to_owned(),
            )),
        )
        .await
        .unwrap()
        .data
        .ship
    }

    pub async fn get_system_waypoints(&self, system_name: &str) -> Vec<models::Waypoint> {
        systems_api::get_system_waypoints(self.configuration, system_name, None, None, None, None)
            .await
            .unwrap()
            .data
    }

    pub async fn get_my_ships(&self) -> Vec<Ship> {
        fleet::get_my_ships(self.configuration, None, None)
            .await
            .unwrap()
            .data
    }

    pub async fn get_ship(&self, ship_symbol: &str) -> Box<Ship> {
        fleet::get_my_ship(self.configuration, ship_symbol)
            .await
            .unwrap()
            .data
    }

    pub async fn dock_ship(&self, ship_symbol: &str) {
        fleet::dock_ship(self.configuration, ship_symbol)
            .await
            .unwrap();
    }

    pub async fn navigate(&self, ship_symbol: &str, waypoint_symbol: &str) {
        let resp = fleet::navigate_ship(
            self.configuration,
            ship_symbol,
            Some(NavigateShipRequest::new(waypoint_symbol.to_owned())),
        )
        .await
        .unwrap()
        .data;

        dbg!(&resp);

        let route = resp.nav.route;
        let departure = DateTime::parse_from_rfc3339(route.departure_time.as_str()).unwrap();
        let arrival = DateTime::parse_from_rfc3339(route.arrival.as_str()).unwrap();

        let eta = (arrival - departure).num_seconds() as u64;

        info!("[{ship_symbol}] Travelling to {waypoint_symbol}, sleeping {eta}");
        tokio::time::sleep(Duration::from_secs(eta)).await;
    }

    pub async fn orbit_ship(&self, ship_symbol: &str) {
        fleet::orbit_ship(self.configuration, ship_symbol)
            .await
            .unwrap();
    }

    pub async fn sell_all(&self, ship_symbol: &str) {
        let cargo = fleet::get_my_ship_cargo(self.configuration, ship_symbol)
            .await
            .unwrap();

        for c in cargo.data.inventory {
            let resp = fleet::sell_cargo(
                self.configuration,
                ship_symbol,
                Some(SellCargoRequest::new(c.symbol, c.units)),
            )
            .await;
            match resp {
                Result::Ok(a) => {
                    let transaction = a.data.transaction;

                    let context = &self.log_context;
                    info!(
                        "[{context}] Sold {}x{} for {} credits. Total credits={}",
                        transaction.units,
                        transaction.trade_symbol.as_str(),
                        transaction.total_price,
                        a.data.agent.credits
                    )
                }
                Result::Err(e) => {
                    let err: GenericError<SellCargoError> = e.into();
                    let context = &self.log_context;
                    match err.error.data {
                        SellCargoError::NotFoundError(cargo) => {
                            info!(
                                "[{context}] Failed to sell cargo. Tried to sell {}x{} but had {}x{}",
                                cargo.units_to_remove,
                                cargo.trade_symbol,
                                cargo.cargo_units,
                                cargo.trade_symbol
                            )
                        }
                        SellCargoError::NotSellableError(sell) => {
                            info!(
                                "[{context}] Failed to sell {}x{} as is not sellable in this market",
                                c.units,
                                sell.trade_symbol,
                            )
                        }
                    }
                }
            }
        }
    }

    pub async fn extract_till_full(&self, ship_symbol: &str) {
        loop {
            let extracted = fleet::extract_resources(
                self.configuration,
                ship_symbol,
                Some(ExtractResourcesRequest::new()),
            )
            .await;

            match extracted {
                Result::Ok(r) => {
                    let cargo = r.data.cargo;
                    let units = cargo.units;
                    let capacity = cargo.capacity;

                    let yld = r.data.extraction.r#yield;
                    let yld_symbol = yld.symbol.to_string();
                    let yld_units = yld.units;

                    let sleep_seconds = r.data.cooldown.remaining_seconds as u64;

                    info!("[{ship_symbol}] extraction cooldown, yield={yld_units}x{yld_symbol}, inventory={units}/{capacity}, sleeping for {sleep_seconds} seconds");

                    if capacity - units < 3 {
                        return;
                    }

                    tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
                }
                Result::Err(e) => {
                    let err: GenericError<ExtractResourceError> = e.into();

                    let sleep_seconds = match err.error.data {
                        ExtractResourceError::Cooldown { cooldown } => cooldown.remaining_seconds,
                        ExtractResourceError::Cargo { .. } => return,
                    };

                    info!(
                        "[{ship_symbol}] extraction cooldown, sleeping for {sleep_seconds} seconds"
                    );

                    tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialise_sell_cargo_error_not_sellable() {
        let str = "{\"error\":{\"message\":\"Market sell failed. Trade good ANTIMATTER is not available at X1-ZA40-15970B.\",\"code\":4602,\"data\":{\"waypointSymbol\":\"X1-ZA40-15970B\",\"tradeSymbol\":\"ANTIMATTER\"}}}";

        let err: GenericError<SellCargoError> = serde_json::from_str(str).unwrap();

        match err.error.data {
            SellCargoError::NotFoundError(_) => panic!(),
            SellCargoError::NotSellableError(e) => {
                assert_eq!(e.waypoint_symbol, "X1-ZA40-15970B".to_owned())
            }
        }
    }

    #[test]
    fn deserialise_sell_cargo_error() {
        let str = "{\"error\":{\"message\":\"Failed t COPPER_ORE.\",\"code\":4219,\"data\":{\"shipSymbol\":\"MXZ-3\",\"tradeSymbol\":\"COPPER_ORE\",\"cargoUnits\":0,\"unitsToRemove\":7}}}";

        let err: GenericError<SellCargoError> = serde_json::from_str(str).unwrap();

        match err.error.data {
            SellCargoError::NotFoundError(cargo) => {
                assert_eq!(cargo.cargo_units, 0)
            }
            SellCargoError::NotSellableError(_) => panic!(),
        }
    }

    #[test]
    fn deserialise_cargo() {
        let str = "{\"error\":{\"message\":\"Failed of available space.\",\"code\":4228,\"data\":{\"shipSymbol\":\"MXZ-2\"}}}";

        let err: GenericError<ExtractResourceError> = serde_json::from_str(str).unwrap();

        match err.error.data {
            ExtractResourceError::Cargo { .. } => (),
            ExtractResourceError::Cooldown { .. } => panic!(),
        }
    }

    #[test]
    fn deserialise_cooldown() {
        let str = "{\"error\":{\"message\":\"cooldown.\",\"code\":4000,\"data\":{\"cooldown\":{\"shipSymbol\":\"MXZ-2\", \"expiration\":\"bla\", \"remainingSeconds\": 5, \"totalSeconds\": 10}}}}";

        let err: GenericError<ExtractResourceError> = serde_json::from_str(str).unwrap();

        match err.error.data {
            ExtractResourceError::Cargo { .. } => panic!(),
            ExtractResourceError::Cooldown { cooldown, .. } => {
                assert_eq!(cooldown.remaining_seconds, 5);
            }
        }
    }
}

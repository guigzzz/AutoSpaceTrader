use std::time::Duration;

use chrono::{Date, DateTime, Utc};
use spacedust::{
    apis::{
        agents_api,
        configuration::Configuration,
        fleet_api::{self as fleet},
        Error,
    },
    models::{Agent, ExtractResourcesRequest, NavigateShipRequest, SellCargoRequest, Ship},
};

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::configuration::ConfigBuilder;

#[repr(u16)]
#[derive(Debug, PartialEq, Deserialize_repr)]
pub enum ErrorCode {
    CooldownConflictError = 4000,
    ShipCargoFullError = 4228,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ExtractResourceError {
    Cooldown { cooldown: CoolDownErrorInner },
    Cargo(CargoErrorInner),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericErrorInner<T> {
    code: ErrorCode,
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

pub struct Client {
    configuration: Configuration,
}

impl Client {
    pub fn new() -> Self {
        Self {
            configuration: ConfigBuilder::new_config(),
        }
    }

    pub async fn get_my_agent(&self) -> Box<Agent> {
        agents_api::get_my_agent(&self.configuration)
            .await
            .unwrap()
            .data
    }

    pub async fn get_my_ships(&self) -> Vec<Ship> {
        fleet::get_my_ships(&self.configuration, None, None)
            .await
            .unwrap()
            .data
    }

    pub async fn get_ship(&self, ship_symbol: &str) -> Box<Ship> {
        fleet::get_my_ship(&self.configuration, ship_symbol)
            .await
            .unwrap()
            .data
    }

    pub async fn dock_ship(&self, ship_symbol: &str) {
        fleet::dock_ship(&self.configuration, ship_symbol, 0.)
            .await
            .unwrap();
    }

    pub async fn navigate(&self, ship_symbol: &str, waypoint_symbol: &str) {
        let resp = fleet::navigate_ship(
            &self.configuration,
            ship_symbol,
            Some(NavigateShipRequest::new(waypoint_symbol.to_owned())),
        )
        .await
        .unwrap()
        .data;

        dbg!(&resp);

        let route = resp.nav.route;
        let departure = DateTime::parse_from_rfc3339(route.arrival.as_str()).unwrap();
        let arrival = DateTime::parse_from_rfc3339(route.departure_time.as_str()).unwrap();

        let eta = arrival - departure;

        println!("[{ship_symbol}] Travelling to {waypoint_symbol}, sleeping {eta}");
        tokio::time::sleep(Duration::from_secs(eta.num_seconds() as u64)).await;
    }

    pub async fn orbit_ship(&self, ship_symbol: &str) {
        fleet::orbit_ship(&self.configuration, ship_symbol, 0)
            .await
            .unwrap();
    }

    pub async fn sell_all(&self, ship_symbol: &str) {
        let cargo = fleet::get_my_ship_cargo(&self.configuration, ship_symbol)
            .await
            .unwrap();

        for c in cargo.data.inventory {
            fleet::sell_cargo(
                &self.configuration,
                ship_symbol,
                Some(SellCargoRequest::new(c.symbol, c.units)),
            )
            .await
            .unwrap();
        }
    }

    pub async fn extract_till_full(&self, ship_symbol: &str) {
        loop {
            let extracted = fleet::extract_resources(
                &self.configuration,
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
                    let yld_symbol = yld.symbol;
                    let yld_units = yld.units;

                    let sleep_seconds = r.data.cooldown.remaining_seconds as u64;

                    println!("[{ship_symbol}] extraction cooldown, yield={yld_units}x{yld_symbol}, inventory={units}/{capacity}, sleeping for {sleep_seconds} seconds");

                    tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
                }
                Result::Err(e) => match e {
                    Error::ResponseError(e) => {
                        let err: GenericError<ExtractResourceError> =
                            match serde_json::from_str(&e.content) {
                                Result::Ok(v) => v,
                                Result::Err(ser_err) => {
                                    dbg!(e);
                                    dbg!(ser_err);
                                    panic!()
                                }
                            };

                        let sleep_seconds = match err.error.data {
                            ExtractResourceError::Cooldown { cooldown } => {
                                cooldown.remaining_seconds
                            }
                            ExtractResourceError::Cargo { .. } => return,
                        };

                        println!("[{ship_symbol}] extraction cooldown, sleeping for {sleep_seconds} seconds");

                        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
                    }
                    _ => panic!(),
                },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

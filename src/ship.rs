use std::time::Duration;

use spacedust::{
    apis::{
        configuration::Configuration,
        fleet_api::{self as fleet},
        Error,
    },
    models::{ExtractResourcesRequest, SellCargoRequest},
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericErrorInner<T> {
    code: u64,
    data: T,
    message: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenericError<T> {
    error: GenericErrorInner<T>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoolDownErrorInner {
    expiration: String,
    remaining_seconds: u64,
    ship_symbol: String,
    total_seconds: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CoolDownError {
    cooldown: CoolDownErrorInner,
}

pub struct Ship {}

impl Ship {
    pub async fn sell_all(config: &Configuration, ship_symbol: &str) {
        let cargo = fleet::get_my_ship_cargo(config, ship_symbol).await.unwrap();

        for c in cargo.data.inventory {
            fleet::sell_cargo(
                config,
                ship_symbol,
                Some(SellCargoRequest::new(c.symbol, c.units)),
            )
            .await
            .unwrap();
        }
    }

    pub async fn is_cargo_full(config: &Configuration, ship_symbol: &str) -> bool {
        let cargo = fleet::get_my_ship_cargo(config, ship_symbol).await.unwrap();
        cargo.data.capacity == cargo.data.units
    }

    pub async fn extract_till_full(config: &Configuration, ship_symbol: &str) {
        while !Self::is_cargo_full(config, ship_symbol).await {
            let extracted =
                fleet::extract_resources(config, ship_symbol, Some(ExtractResourcesRequest::new()))
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
                        let err: GenericError<CoolDownError> =
                            match serde_json::from_str(&e.content) {
                                Result::Ok(v) => v,
                                Result::Err(e) => {
                                    dbg!(e);
                                    panic!()
                                }
                            };

                        let sleep_seconds = err.error.data.cooldown.remaining_seconds;

                        println!("[{ship_symbol}] extraction cooldown, sleeping for {sleep_seconds} seconds");

                        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
                    }
                    _ => panic!(),
                },
            };
        }
    }
}

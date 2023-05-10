use std::env;

use dotenv::dotenv;
use reqwest::ClientBuilder;
use spacedust::apis::configuration::Configuration;

use crate::limiter::RateLimiter;

pub struct ConfigBuilder {}

impl ConfigBuilder {
    pub fn new_config() -> Configuration {
        dotenv().ok();

        let client = reqwest_middleware::ClientBuilder::new(ClientBuilder::new().build().unwrap())
            .with(RateLimiter::new())
            .build();

        Configuration {
            bearer_access_token: Some(env::var("TOKEN").unwrap()),
            client,
            ..Default::default()
        }
    }
}

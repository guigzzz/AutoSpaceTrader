use std::env;

use dotenv::dotenv;
use lazy_static::lazy_static;
use reqwest::{header::HeaderValue, ClientBuilder, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use spacedust::apis::configuration::Configuration;
use task_local_extensions::Extensions;

use crate::limiter::RateLimiter;

pub struct ContentLengthFixMiddleware;

#[async_trait::async_trait]
impl Middleware for ContentLengthFixMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        if req.body().is_none() && !req.headers().contains_key("content-length") {
            req.headers_mut()
                .append("content-length", HeaderValue::from_static("0"));
        }

        next.run(req, extensions).await
    }
}

pub struct ConfigurationFactory {}

impl ConfigurationFactory {
    pub fn get_config(token: &str) -> Configuration {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = reqwest_middleware::ClientBuilder::new(ClientBuilder::new().build().unwrap())
            .with(RateLimiter::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .with(ContentLengthFixMiddleware)
            .build();

        Configuration {
            bearer_access_token: Some(token.to_owned()),
            client,
            ..Default::default()
        }
    }
}

lazy_static! {
    // having a static singleton configuration means that the throttling will apply to any client we instantiate
    pub static ref CONFIGURATION: Configuration = {
        dotenv().ok();
        ConfigurationFactory::get_config(env::var("TOKEN").unwrap().as_str())
    };
}

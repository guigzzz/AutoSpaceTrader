use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use task_local_extensions::Extensions;
use tokio::{sync::Mutex, time::sleep_until};

#[derive(Debug, Default)]
pub struct RateLimiter {
    queue: Mutex<VecDeque<Instant>>,
}

const RATE_TTL: Duration = Duration::from_secs(1);
const RPS: usize = 2;

impl RateLimiter {
    pub fn new() -> Self {
        let queue = VecDeque::new();
        Self {
            queue: Mutex::new(queue),
        }
    }

    async fn sleep_until_allowed(&self) {
        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return;
        }

        while queue.len() >= RPS {
            // Unwraps are safe because len is checked to be non-empty.
            while queue.front().unwrap().elapsed() > RATE_TTL {
                queue.pop_front();
                if queue.is_empty() {
                    return;
                }
            }

            // Skip sleep if last second contains allowed RPS.
            if queue.len() < RPS {
                break;
            }

            // Sleep until at least one more item can be evicted.
            let oldest = queue.front().unwrap();
            sleep_until((*oldest + RATE_TTL).into()).await;
        }
    }

    async fn next_request(&self) {
        self.sleep_until_allowed().await;
        self.queue.lock().await.push_back(Instant::now());
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        self.next_request().await;

        next.run(req, extensions).await
    }
}

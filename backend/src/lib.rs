use napi_derive::napi;
use tokio::time::{Duration, sleep};

#[napi]
pub async fn delayed_sum(a: i32, b: i32) -> i32 {
    sleep(Duration::from_secs(2)).await;
    a + b
}

use std::time::Duration;

use rand::Rng;

pub mod anthropic;
pub mod bedrock;
pub mod gemini;
pub mod jawn;
pub mod minio;
pub mod openai;

pub(crate) async fn sleep(latency: u32) {
    let n: u32 = {
        let mut rng = rand::rng();
        let delta = 10;
        let range = (latency - delta)..(latency + delta);
        rng.random_range(range)
    };
    tokio::time::sleep(Duration::from_millis(n.into())).await;
}

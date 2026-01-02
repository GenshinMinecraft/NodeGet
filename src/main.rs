#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::await_holding_lock,
    dead_code
)]

use crate::monitoring::data_structure::MonitoringData;

mod monitoring;

#[tokio::main]
async fn main() {
    loop {
        let all = MonitoringData::refresh_and_get().await;
        println!("{all:#?}");
        println!("Size: {} Bytes", size_of_val(&all));
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

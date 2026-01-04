#![feature(duration_millis_float)]
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

// use crate::monitoring::data_structure::StaticMonitoringData;
// use crate::utils::get_local_timestamp_ms;
// use sea_orm::*;
// use uuid::uuid;

mod monitoring;
mod utils;

#[tokio::main]
async fn main() {
    // let db_url = "sqlite://test.db?mode=rwc";
    // let db = Database::connect(db_url).await.unwrap();
    //
    // Migrator::up(&db, None).await.unwrap();
    // println!("Migration completed!");
    //
    // loop {
    //     let sta_tic = StaticMonitoringData::get().await;
    //     let active_model = entities::static_monitoring::ActiveModel {
    //         node_uuid: Set(uuid!("5acc7a90-8afb-485b-85d0-d20720f29432")),
    //         time: Set(get_local_timestamp_ms() as i64),
    //         data: Set(serde_json::to_value(sta_tic).unwrap()),
    //         ..Default::default()
    //     };
    //
    //     active_model.insert(&db).await.unwrap();
    //
    //     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // }
}

mod task;

use log::info;
use std::str::FromStr;
use chrono::{TimeZone, Utc};
use cron::Schedule;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, Set};
use sea_orm::{EntityTrait, QueryFilter};
use log::{error, warn};
use nodeget_lib::crontab::{AgentCronType, Cron, CronType};
use crate::DB;
use crate::entity::crontab;
use crate::entity::crontab::Model;

async fn process_crontab() {
    let db = match DB.get() {
        None => {
            error!("DB not initialized");
            return;
        }
        Some(db) => db
    };

    let jobs = match crontab::Entity::find()
        .filter(crontab::Column::Enable.eq(true))
        .all(db)
        .await {
        Ok(jobs) => jobs,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    let now  = Utc::now();

    for job in jobs {
        let schedule = match Schedule::from_str(&job.cron_expression) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid cron expression for job {}: {}", job.id, e);
                continue;
            }
        };

        let last_run = job
            .last_run_time
            .map(|t| Utc.timestamp_millis_opt(t).unwrap())
            .unwrap_or_else(|| now - chrono::Duration::seconds(1));

        if let Some(next_run) = schedule.after(&last_run).next() {
            if next_run <= now {
                info!("Triggering cron job: {} ({})", job.name, job.id);

                let mut active_model: crontab::ActiveModel = job.clone().into();
                active_model.last_run_time = Set(Some(now.timestamp_millis()));
                if let Err(e) = active_model.update(db).await {
                    error!("Failed to update last_run_time for job {}: {}", job.id, e);
                    continue;
                }

                let job_parsed = Cron {
                    id: job.id,
                    name: job.name,
                    enable: job.enable,
                    cron_expression: job.cron_expression,
                    cron_type: {
                        match serde_json::from_str(&job.cron_type.to_string()) {
                            Ok(cron_type) => cron_type,
                            Err(e) => {
                                warn!("Invalid cron type for job {}: {}", job.id, e);
                                continue;
                            }
                        }
                    },
                    last_run_time: job.last_run_time,
                };

                tokio::spawn(async move {
                    run_job_logic(job_parsed).await;
                });
            }
        }
    }
}

async fn run_job_logic(job: Cron) {
    match job.cron_type {
        CronType::Agent(uuids, agent_cron) => {
            match agent_cron {
                AgentCronType::Task(task_event_type) => {
                    todo!()
                }
            }
        }
        CronType::Server(_) => {
            todo!()
        }
    }
}
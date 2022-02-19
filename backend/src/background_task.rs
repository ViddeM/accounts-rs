use crate::db::{
    account_repository, activation_code_repository, login_details_repository, new_transaction, DB,
};
use crate::util::accounts_error::AccountsResult;
use futures::executor::block_on;
use job_scheduler::{Job, JobScheduler, Schedule};
use sqlx::Pool;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub enum BgTaskError {
    JobSetupFailed,
}

pub async fn run_background_tasks(db_pool: Pool<DB>) -> Result<(), BgTaskError> {
    let mut sched = JobScheduler::new();

    // Run task every half hour
    sched.add(Job::new(
        Schedule::from_str("0 0,30 * * * * *").or(Err(BgTaskError::JobSetupFailed))?,
        || {
            block_on(delete_unactived_accounts(db_pool.clone()))
                .expect("Failed to delete unactivated accounts");
        },
    ));

    println!("Background jobs started, starting check for job executions");

    // Keep running the jobs forever
    loop {
        sched.tick();
        std::thread::sleep(sched.time_till_next_job());
    }
}

// 12 hours
const MINUTES_TO_ACTIVATE_ACCOUNT: u64 = 60 * 12;

async fn delete_unactived_accounts(db_pool: Pool<DB>) -> AccountsResult<()> {
    let mut transaction = new_transaction(&db_pool).await?;

    let outdated_codes =
        activation_code_repository::delete_outdated(&mut transaction, MINUTES_TO_ACTIVATE_ACCOUNT)
            .await?;

    let unactivated_account_ids: Vec<Uuid> = outdated_codes
        .into_iter()
        .map(|code| code.login_details)
        .collect();

    login_details_repository::delete_multiple(&mut transaction, unactivated_account_ids.as_slice())
        .await?;
    account_repository::delete_multiple(&mut transaction, unactivated_account_ids.as_slice())
        .await?;

    transaction.commit().await?;

    println!(
        "Delete unactivated accounts job ran successfully, deleted {}",
        unactivated_account_ids.len()
    );

    Ok(())
}

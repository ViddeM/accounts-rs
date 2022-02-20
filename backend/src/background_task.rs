use crate::db::{
    account_repository, activation_code_repository, login_details_repository, new_transaction, DB,
};
use crate::util::accounts_error::AccountsResult;
use sqlx::Pool;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const SECONDS_BETWEEN_TASKS: u64 = 30 * 60 * 60;

pub async fn run_background_tasks(db_pool: Pool<DB>) {
    let time_between_tasks = Duration::from_secs(SECONDS_BETWEEN_TASKS);

    // Keep running the jobs forever
    loop {
        delete_unactived_accounts(&db_pool)
            .await
            .expect("Failed to delete unactivated accounts");
        println!(
            "Run successful, sleeping for {} minutes",
            time_between_tasks.as_secs() / 60
        );
        time::sleep(time_between_tasks).await;
    }
}

// 12 hours
const MINUTES_TO_ACTIVATE_ACCOUNT: u64 = 60 * 12;

async fn delete_unactived_accounts(db_pool: &Pool<DB>) -> AccountsResult<()> {
    println!("Begin deletion of unactivated accounts");

    let mut transaction = new_transaction(&db_pool).await?;

    let outdated_codes =
        activation_code_repository::delete_outdated(&mut transaction, MINUTES_TO_ACTIVATE_ACCOUNT)
            .await?;
    println!(
        "Deleted outdated codes {:?} using {}",
        outdated_codes, MINUTES_TO_ACTIVATE_ACCOUNT
    );

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

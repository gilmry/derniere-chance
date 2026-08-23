use crate::infrastructure::database::DbPool;

/// If no admin account exists yet, creates one from the `ADMIN_EMAIL` /
/// `ADMIN_PASSWORD` env vars (same pattern as Elevia's bootstrap_admin).
///
/// There is no public endpoint to create the first admin - the backoffice
/// is a single, non-self-service account - so without this, a fresh
/// deployment would have no way to log in at all.
pub async fn bootstrap_admin(pool: &DbPool) {
    let (has_admin,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM admins)")
        .fetch_one(pool)
        .await
        .expect("failed to check for an existing admin account");

    if has_admin {
        return;
    }

    let (Ok(email), Ok(password)) =
        (std::env::var("ADMIN_EMAIL"), std::env::var("ADMIN_PASSWORD"))
    else {
        tracing::warn!(
            "no admin account exists and ADMIN_EMAIL/ADMIN_PASSWORD are not set - \
             nobody will be able to log in to the backoffice until an admin is bootstrapped"
        );
        return;
    };

    let password_hash =
        bcrypt::hash(&password, bcrypt::DEFAULT_COST).expect("failed to hash admin password");

    sqlx::query("INSERT INTO admins (email, password_hash) VALUES ($1, $2)")
        .bind(&email)
        .bind(password_hash)
        .execute(pool)
        .await
        .expect("failed to create the bootstrap admin account");

    tracing::info!(%email, "created initial backoffice admin account from ADMIN_EMAIL/ADMIN_PASSWORD");
}

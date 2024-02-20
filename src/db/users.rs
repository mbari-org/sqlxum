use crate::common::unescape_query;
use crate::models::User;
use crate::server::database::{UserPostReq, UserPutReq};

pub async fn do_users_query(pool: &sqlx::PgPool, query: &str) -> anyhow::Result<Vec<User>> {
    let query = unescape_query(query);
    println!("query: {}\n", query);

    Ok(sqlx::query_as::<_, User>(&query).fetch_all(pool).await?)
}

pub async fn insert_user(pool: &sqlx::PgPool, req: &UserPostReq) -> anyhow::Result<User> {
    let record = sqlx::query!(
        r#"
            insert into usr (email, name) values ($1, $2)
            returning user_id, email, name, created_at, updated_at
        "#,
        req.email,
        req.name,
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        user_id: record.user_id,
        email: record.email.clone(),
        name: record.name.clone(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

pub async fn update_user(pool: &sqlx::PgPool, req: &UserPutReq) -> anyhow::Result<User> {
    let record = sqlx::query!(
        r#"
            update usr
             set email = coalesce($1, usr.email),
                  name = coalesce($2, usr.name)
            where user_id = $3
            returning user_id, email, name, created_at, updated_at
        "#,
        req.email,
        req.name,
        req.user_id.into(),
    )
    .fetch_one(pool)
    .await?;

    Ok(User {
        user_id: record.user_id,
        email: record.email.clone(),
        name: record.name.clone(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    })
}

pub async fn delete_user_by_id(
    pool: &sqlx::PgPool,
    user_id: &uuid::Uuid,
) -> anyhow::Result<uuid::Uuid> {
    let record = sqlx::query!(
        r#"
            delete from usr where user_id = $1
            returning user_id
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(record.user_id)
}

pub async fn delete_user_by_email(pool: &sqlx::PgPool, email: &str) -> anyhow::Result<uuid::Uuid> {
    let record = sqlx::query!(
        r#"
            delete from usr where email = $1
            returning user_id
        "#,
        email,
    )
    .fetch_one(pool)
    .await?;
    Ok(record.user_id)
}

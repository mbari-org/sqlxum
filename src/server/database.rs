use std::time::Instant;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{IntoParams, ToSchema};

use crate::db::generic;
use crate::db::users;
use crate::models::User;
use crate::server::AppState;

pub async fn create_router(app_state: AppState) -> anyhow::Result<Router> {
    Ok(Router::new()
        .route("/query", post(do_query))
        .route(
            "/users",
            get(get_users)
                .put(update_user)
                .post(add_user)
                .delete(delete_user),
        )
        .with_state(app_state))
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct QueryReq {
    /// The query to execute.
    query: String,
}

/// Perform a database query.
#[utoipa::path(
    post,
    path = "/query",
    request_body = QueryReq,
    responses(
       (status = 200, description = "Query response", body = Value)
    )
)]
pub async fn do_query(state: State<AppState>, Json(req): Json<QueryReq>) -> Json<Value> {
    log::debug!("do_query = {req:?}");
    let pool = &state.pool;
    let query = req.query;
    match generic::do_query(pool, &query).await {
        Ok(res) => Json(res),
        Err(e) => {
            log::error!("Error: {:?}", e);
            Json(json!({
                "query": query,
                "error": e.to_string()
            }))
        }
    }
}

#[derive(Deserialize, Debug, IntoParams)]
pub struct QueryParams {
    /// Where clause. Example: `name = 'Foo'`.
    r#where: Option<String>,

    /// Limit the number of results. By default, some small number.
    limit: Option<u32>,
}

impl QueryParams {
    fn adjust_query(self, query: &str) -> String {
        let mut query = query.to_string();
        if let Some(cond) = &self.r#where {
            query = format!("{query} where {cond}");
        }
        let limit = self.limit.unwrap_or(5);
        format!("{query} limit {limit}")
    }
}

#[derive(Serialize, ToSchema, Debug)]
pub struct UserRes {
    #[schema(value_type = str)]
    pub user_id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl UserRes {
    pub fn from_user(user: &User) -> Self {
        UserRes {
            user_id: user.user_id,
            email: user.email.clone(),
            name: user.name.clone(),
            created_at: user.created_at.to_string(),
            updated_at: user.updated_at.map(|d| d.to_string()),
        }
    }
}

/// Get users
#[utoipa::path(
    get,
    path = "/users",
    params(QueryParams),
    responses(
       (status = 200, description = "List of users", body = Vec<UserRes>)
    )
)]
pub async fn get_users(
    state: State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    log::info!("get_users: {params:?}");
    let query = params.adjust_query("select * from usr");

    let pool = &state.pool;
    let start = Instant::now();
    match users::do_users_query(pool, &query).await {
        Ok(res) => {
            let elapsed = format!("{:?}", start.elapsed());
            let result: Vec<UserRes> = res.iter().map(UserRes::from_user).collect();
            Json(json!({
                "query": query,
                "result": result,
                "elapsed": elapsed,
            }))
            .into_response()
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Request content to add a user
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UserPostReq {
    /// Email of the user
    pub email: String,
    /// Name of the user
    pub name: String,
}

/// Add a user
#[utoipa::path(
    post,
    path = "/users",
    request_body = UserPostReq,
    responses(
       (status = 200, description = "User registered", body = UserRes)
    )
)]
pub async fn add_user(state: State<AppState>, Json(req): Json<UserPostReq>) -> impl IntoResponse {
    log::info!("add_user: {req:?}");
    let pool = &state.pool;
    match users::insert_user(pool, &req).await {
        Ok(user) => {
            let user_res = UserRes::from_user(&user);
            Json(user_res).into_response()
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Request content to update a user
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UserPutReq {
    /// ID of the user
    #[schema(value_type = str)]
    pub user_id: uuid::Uuid,
    /// New email of the user
    pub email: Option<String>,
    /// New name of the user
    pub name: String,
}

/// Update a user
#[utoipa::path(
    put,
    path = "/users",
    request_body = UserPutReq,
    responses(
       (status = 200, description = "User registered", body = UserRes)
    )
)]
pub async fn update_user(state: State<AppState>, Json(req): Json<UserPutReq>) -> impl IntoResponse {
    log::info!("update_user: {req:?}");
    let pool = &state.pool;
    match users::update_user(pool, &req).await {
        Ok(user) => {
            let user_res = UserRes::from_user(&user);
            Json(user_res).into_response()
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

/// Request content to delete a user
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UserDeleteReq {
    /// ID of user to delete. Has precedence over `email`.
    #[schema(value_type = Option<str>)]
    pub user_id: Option<uuid::Uuid>,

    /// Email of user to delete. If `user_id` is present, this is ignored.
    pub email: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct UserDeleteRes {
    #[schema(value_type = str)]
    pub user_id: uuid::Uuid,
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/users",
    request_body = UserDeleteReq,
    responses(
       (status = 200, description = "User deleted", body = UserDeleteRes)
    )
)]
pub async fn delete_user(
    state: State<AppState>,
    Json(req): Json<UserDeleteReq>,
) -> impl IntoResponse {
    log::info!("delete_user: {req:?}");
    if req.user_id.is_none() && req.email.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            "Either user_id or email must be provided",
        )
            .into_response();
    }

    let pool = &state.pool;

    let res = if let Some(user_id) = req.user_id {
        users::delete_user_by_id(pool, &user_id).await
    } else {
        users::delete_user_by_email(pool, &req.email.unwrap()).await
    };
    match res {
        Ok(user_id) => Json(UserDeleteRes { user_id }).into_response(),
        Err(e) => {
            log::error!("Error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{
    error::AppError,
    middleware::auth::{require_role, AuthUser},
    models::user::UserRole,
};

use super::auth::AppState;

pub async fn sse_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    require_role(
        &claims,
        &[
            UserRole::Admin,
            UserRole::Cashier,
            UserRole::Kitchen,
            UserRole::Waiter,
        ],
    )?;

    let rx = state.sse.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| match res {
        Ok(event) => {
            let json_str = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json_str)))
        }
        Err(_) => None,
    });

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}

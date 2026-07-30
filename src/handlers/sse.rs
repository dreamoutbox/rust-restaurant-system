use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures_util::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, require_role},
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

    let initial =
        futures_util::stream::once(async { Ok(Event::default().data(r#"{"type":"connected"}"#)) });

    let broadcast_stream = BroadcastStream::new(rx).filter_map(|res| match res {
        Ok(event) => {
            let json_str = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json_str)))
        }
        Err(_) => None,
    });

    let combined_stream = initial.chain(broadcast_stream);

    Ok(Sse::new(combined_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("ping"),
    ))
}

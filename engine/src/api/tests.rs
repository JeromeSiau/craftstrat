use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use super::state::ApiState;

fn test_state() -> Arc<ApiState> {
    Arc::new(ApiState {
        registry: crate::strategy::registry::AssignmentRegistry::new(),
        exec_queue: Arc::new(tokio::sync::Mutex::new(
            crate::execution::queue::ExecutionQueue::new(100),
        )),
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test@localhost/test")
            .unwrap(),
        ch: clickhouse::Client::default(),
        redis: None,
        start_time: std::time::Instant::now(),
        tick_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
    })
}

#[tokio::test]
async fn test_activate_then_deactivate() {
    let state = test_state();
    let app = super::router(state.clone());

    // Activate
    let body = serde_json::json!({
        "wallet_id": 1,
        "strategy_id": 100,
        "graph": {"mode": "form"},
        "markets": ["btc-15m"]
    });
    let req = Request::builder()
        .method("POST")
        .uri("/internal/strategy/activate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify registry
    let reg = state.registry.read().await;
    assert!(reg.contains_key("btc-15m"));
    let assignments = &reg["btc-15m"];
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].wallet_id, 1);
    assert_eq!(assignments[0].strategy_id, 100);
    drop(reg);

    // Deactivate
    let body = serde_json::json!({"wallet_id": 1, "strategy_id": 100});
    let req = Request::builder()
        .method("POST")
        .uri("/internal/strategy/deactivate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let reg = state.registry.read().await;
    assert!(!reg.contains_key("btc-15m"));
}

#[tokio::test]
async fn test_wallet_state_empty() {
    let state = test_state();
    let app = super::router(state);

    let req = Request::builder()
        .uri("/internal/wallet/999/state")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["wallet_id"], 999);
    assert_eq!(json["assignments"], serde_json::json!([]));
}

#[tokio::test]
async fn test_wallet_state_with_assignment() {
    let state = test_state();

    crate::strategy::registry::activate(
        &state.registry,
        42,
        200,
        serde_json::json!({"mode": "form"}),
        vec!["btc-15m".into()],
        500.0,
        None,
    )
    .await;

    let app = super::router(state);
    let req = Request::builder()
        .uri("/internal/wallet/42/state")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["wallet_id"], 42);
    assert_eq!(json["assignments"].as_array().unwrap().len(), 1);
    assert_eq!(json["assignments"][0]["strategy_id"], 200);
}

#[tokio::test]
async fn test_engine_status() {
    let state = test_state();
    state
        .tick_count
        .store(42000, std::sync::atomic::Ordering::Relaxed);

    crate::strategy::registry::activate(
        &state.registry,
        1,
        100,
        serde_json::json!({}),
        vec!["btc-15m".into()],
        100.0,
        None,
    )
    .await;
    crate::strategy::registry::activate(
        &state.registry,
        2,
        200,
        serde_json::json!({}),
        vec!["btc-15m".into()],
        100.0,
        None,
    )
    .await;

    let app = super::router(state);
    let req = Request::builder()
        .uri("/internal/engine/status")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["active_wallets"], 2);
    assert_eq!(json["active_assignments"], 2);
    assert_eq!(json["ticks_processed"], 42000);
}

#[tokio::test]
async fn test_copy_watch_unavailable_without_redis() {
    let state = test_state();
    let app = super::router(state);

    let body = serde_json::json!({"leader_address": "0xabc"});
    let req = Request::builder()
        .method("POST")
        .uri("/internal/copy/watch")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_copy_unwatch_unavailable_without_redis() {
    let state = test_state();
    let app = super::router(state);

    let body = serde_json::json!({"leader_address": "0xabc"});
    let req = Request::builder()
        .method("POST")
        .uri("/internal/copy/unwatch")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_activate_empty_markets_returns_validation_error() {
    let state = test_state();
    let app = super::router(state);

    let body = serde_json::json!({
        "wallet_id": 1,
        "strategy_id": 100,
        "graph": {"mode": "form"},
        "markets": []
    });
    let req = Request::builder()
        .method("POST")
        .uri("/internal/strategy/activate")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("markets"));
}

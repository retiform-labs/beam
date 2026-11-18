#![cfg(not(target_arch = "wasm32"))]
//! Minimal repro: does `batch_put` persist what `put` persists?

use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::timeout;

use beam::sentinel::REPLAY_COMPLETE;
use beam::{Node, Value};

const T: Duration = Duration::from_secs(3);

async fn drain(rx: &mut tokio::sync::broadcast::Receiver<(String, Value)>) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    loop {
        match timeout(T, rx.recv()).await {
            Ok(Ok((k, v))) => {
                if k == REPLAY_COMPLETE {
                    break;
                }
                out.push((k, v));
            }
            Ok(Err(RecvError::Lagged(_))) => continue,
            Ok(Err(_)) | Err(_) => break,
        }
    }
    out
}

fn keys(items: &[(String, Value)]) -> Vec<String> {
    items.iter().map(|(k, _)| k.clone()).collect()
}

fn path() -> Vec<String> {
    vec!["alpha".into(), "beta".into(), "gamma".into()]
}

#[tokio::test]
async fn control_put_visible_on_parent_map() {
    let mut db = Node::new();
    db.get("alpha")
        .get("beta")
        .get("gamma")
        .put(Value::Text("hello".into()))
        .await
        .unwrap();
    let mut rx = db.get("alpha").get("beta").map();
    let items = drain(&mut rx).await;
    db.stop();
    assert!(
        keys(&items).contains(&"gamma".to_string()),
        "put: parent map got {:?}",
        keys(&items)
    );
}

#[tokio::test]
async fn suspect_batch_put_visible_on_parent_map() {
    let mut db = Node::new();
    db.batch_put(vec![(path(), Value::Text("hello".into()))])
        .await
        .unwrap();
    let mut rx = db.get("alpha").get("beta").map();
    let items = drain(&mut rx).await;
    db.stop();
    assert!(
        keys(&items).contains(&"gamma".to_string()),
        "batch_put: parent map got {:?}",
        keys(&items)
    );
}

#[tokio::test]
async fn control_put_visible_on_leaf_map() {
    let mut db = Node::new();
    db.get("alpha")
        .get("beta")
        .get("gamma")
        .put(Value::Text("hello".into()))
        .await
        .unwrap();
    let mut rx = db.get("alpha").get("beta").get("gamma").map();
    let items = drain(&mut rx).await;
    db.stop();
    assert!(
        keys(&items).contains(&"_".to_string()),
        "put: leaf map got {:?}",
        keys(&items)
    );
}

#[tokio::test]
async fn suspect_batch_put_visible_on_leaf_map() {
    let mut db = Node::new();
    db.batch_put(vec![(path(), Value::Text("hello".into()))])
        .await
        .unwrap();
    let mut rx = db.get("alpha").get("beta").get("gamma").map();
    let items = drain(&mut rx).await;
    db.stop();
    assert!(
        keys(&items).contains(&"_".to_string()),
        "batch_put: leaf map got {:?}",
        keys(&items)
    );
}

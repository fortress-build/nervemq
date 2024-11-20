use std::collections::HashMap;
use std::ops::Deref;

use actix_identity::Identity;
use nervemq::{
    api::auth::Role,
    config::Config,
    db::{namespace::Namespace, queue::Queue},
    service::Service,
};
use tempfile::TempDir;

struct TmpService {
    svc: Service,
    #[allow(unused)]
    tmpdir: TempDir,
}

impl Deref for TmpService {
    type Target = Service;

    fn deref(&self) -> &Self::Target {
        &self.svc
    }
}

// Setup a temporary service
async fn setup() -> TmpService {
    let path = tempfile::tempdir().unwrap();

    let svc = Service::connect_with(Config {
        db_path: Some(path.path().join("nervemq.db").to_string_lossy().to_string()),
    })
    .await
    .unwrap();

    svc.create_user(
        "test@user.com".try_into().unwrap(),
        "abcd1234".into(),
        Some(Role::Admin),
        vec![],
    )
    .await
    .unwrap();

    TmpService { svc, tmpdir: path }
}

fn mock_id() -> Identity {
    Identity::mock("test@user.com".into())
}

#[tokio::test]
async fn test_list_namespace() {
    let service = setup().await;

    let id = mock_id();
    assert_eq!(service.list_namespaces(id).await.unwrap(), vec![]);

    //create namespace to test
    let id = mock_id();
    service.create_namespace("testing", id).await.unwrap();

    //checks to see if the list_namespace function contains the newly created namespace
    let id = mock_id();
    assert_eq!(
        service.list_namespaces(id).await.unwrap(),
        vec![Namespace {
            id: 1,
            name: "testing".to_owned(),
            created_by: "test@user.com".to_owned()
        }]
    );
}

#[tokio::test]
async fn test_delete_namespace() {
    let service = setup().await;

    let id = mock_id();

    assert_eq!(service.list_namespaces(id).await.unwrap(), vec![]);

    let id = mock_id();
    //create namespace
    service.create_namespace("testing", id).await.unwrap();
    //checks to see if namespace is created
    let id = mock_id();
    assert_eq!(
        service.list_namespaces(id).await.unwrap(),
        vec![Namespace {
            id: 1,
            name: "testing".to_owned(),
            created_by: "test@user.com".into()
        }]
    );

    let id = mock_id();
    //delete namespace
    service.delete_namespace("testing", id).await.unwrap();

    let id = mock_id();
    //checks to see if namespace is deleted
    assert_eq!(service.list_namespaces(id).await.unwrap(), vec![]);
}

#[tokio::test]
async fn test_delete_queue() {
    let service = setup().await;

    let id = mock_id();

    // Create namespace first
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    // Verify no queues exist initially
    assert_eq!(
        service.list_queues(Some("testing"), id).await.unwrap(),
        vec![]
    );

    let id = mock_id();
    // Create queue
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();

    let id = mock_id();
    // Verify queue exists
    assert_eq!(
        service.list_queues(Some("testing"), id).await.unwrap(),
        vec![Queue {
            id: 1,
            name: "test-queue".to_owned(),
            ns: "testing".to_owned(),
            created_by: "test@user.com".to_owned()
        }]
    );

    let id = mock_id();
    // Delete queue
    service
        .delete_queue("testing", "test-queue", id)
        .await
        .unwrap();

    let id = mock_id();
    // Verify queue is deleted
    assert_eq!(
        service.list_queues(Some("testing"), id).await.unwrap(),
        vec![]
    );
}

#[tokio::test]
async fn test_list_queues() {
    let service = setup().await;

    let id = mock_id();

    // Create namespace
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    // Verify empty initially
    assert_eq!(
        service.list_queues(Some("testing"), id).await.unwrap(),
        vec![]
    );

    let id = mock_id();
    // Create queue
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();
    // Verify queue is listed
    let id = mock_id();
    assert_eq!(
        service.list_queues(Some("testing"), id).await.unwrap(),
        vec![Queue {
            id: 1,
            name: "test-queue".to_owned(),
            ns: "testing".to_owned(),
            created_by: "test@user.com".into()
        }]
    );
}

#[tokio::test]
async fn test_send_message() {
    let service = setup().await;

    let id = mock_id();

    // Setup namespace and queue
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();

    // Send a message with empty HashMap
    let message = "Hello, World!".as_bytes().to_vec();
    let kv = HashMap::new();
    service
        .send_message("testing", "test-queue", &message, kv)
        .await
        .unwrap();

    // Verify message exists
    let messages = service
        .list_messages("testing", "test-queue")
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, message);
}

#[tokio::test]
async fn test_list_messages() {
    let service = setup().await;
    let id = mock_id();

    // Setup namespace and queue
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();

    // Send a message
    let message = "Hello, World!".as_bytes().to_vec();
    let kv = HashMap::new();
    service
        .send_message("testing", "test-queue", &message, kv)
        .await
        .unwrap();

    // Verify message exists
    let messages = service
        .list_messages("testing", "test-queue")
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, message);
}

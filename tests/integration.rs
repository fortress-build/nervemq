use std::ops::Deref;
use std::{collections::HashMap, rc::Rc};

use actix_identity::Identity;
use nervemq::namespace::Namespace;
use nervemq::queue::Queue;
use nervemq::{api::auth::Role, config::Config, service::Service};
use tempfile::TempDir;

#[derive(Clone)]
struct TmpService {
    svc: Service,
    #[allow(unused)]
    tmpdir: Rc<TempDir>,
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
        default_max_retries: None,
        root_email: None,
        root_password: None,
        host: "http://localhost:8080".try_into().unwrap(),
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

    TmpService {
        svc,
        tmpdir: Rc::new(path),
    }
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
            created_by: "test@user.com".to_owned(),
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
            created_by: "test@user.com".into(),
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
    let queue_id = service
        .get_queue_id(
            "testing",
            "test-queue",
            &mut service.db().acquire().await.unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    service.sqs_send(queue_id, &message, kv).await.unwrap();

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
        .sqs_send(
            service
                .get_queue_id(
                    "testing",
                    "test-queue",
                    &mut service.db().acquire().await.unwrap(),
                )
                .await
                .unwrap()
                .unwrap(),
            &message,
            kv,
        )
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
async fn test_batch_send_and_receive() {
    let service = setup().await;
    let id = mock_id();

    // Setup namespace and queue
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();

    // Prepare batch of messages
    let messages = vec![
        ("Message 1".as_bytes().to_vec(), HashMap::new()),
        ("Message 2".as_bytes().to_vec(), HashMap::new()),
        ("Message 3".as_bytes().to_vec(), HashMap::new()),
    ];

    // Send batch
    let message_ids = service
        .sqs_send_batch("testing", "test-queue", messages.clone())
        .await
        .unwrap();

    assert_eq!(message_ids.len(), 3);

    // Receive batch with smaller size than sent
    let received = service
        .sqs_recv_batch("testing", "test-queue", 2)
        .await
        .unwrap();

    assert_eq!(received.len(), 2);
    assert_eq!(received[0].body, messages[0].0);
    assert_eq!(received[1].body, messages[1].0);
    assert!(received.iter().all(|m| m.delivered_at.is_some()));

    // Receive remaining message
    let received = service
        .sqs_recv_batch("testing", "test-queue", 2)
        .await
        .unwrap();

    assert_eq!(received.len(), 1);
    assert_eq!(received[0].body, messages[2].0);
    assert!(received[0].delivered_at.is_some());

    // Verify no more messages
    let received = service
        .sqs_recv_batch("testing", "test-queue", 10)
        .await
        .unwrap();
    assert!(received.is_empty());
}

#[tokio::test]
async fn test_concurrent_batch_recv() {
    let service = setup().await;
    let id = mock_id();

    // Setup namespace and queue
    service.create_namespace("testing", id).await.unwrap();

    let id = mock_id();
    service
        .create_queue("testing", "test-queue", id)
        .await
        .unwrap();

    // Prepare and send 10 messages
    let messages: Vec<_> = (0..10)
        .map(|i| (format!("Message {}", i).as_bytes().to_vec(), HashMap::new()))
        .collect();

    service
        .sqs_send_batch("testing", "test-queue", messages)
        .await
        .unwrap();

    // Spawn three concurrent batch receivers
    let service1 = service.clone();
    let service2 = service.clone();
    let service3 = service.clone();

    let (batch1, batch2, batch3) = tokio::join!(
        service1.sqs_recv_batch("testing", "test-queue", 4),
        service2.sqs_recv_batch("testing", "test-queue", 4),
        service3.sqs_recv_batch("testing", "test-queue", 4)
    );

    let batch1 = batch1.unwrap();
    let batch2 = batch2.unwrap();
    let batch3 = batch3.unwrap();

    // Verify we got all messages with no duplicates
    assert_eq!(batch1.len() + batch2.len() + batch3.len(), 10);

    // Collect all message IDs
    let mut received_ids: Vec<_> = batch1.iter().map(|m| m.id).collect();
    received_ids.extend(batch2.iter().map(|m| m.id));
    received_ids.extend(batch3.iter().map(|m| m.id));
    received_ids.sort();

    // Verify no duplicates
    let mut deduped = received_ids.clone();
    deduped.dedup();
    assert_eq!(received_ids, deduped);

    // Verify all messages were marked as delivered
    let messages = service
        .list_messages("testing", "test-queue")
        .await
        .unwrap();
    assert!(messages.iter().all(|m| m.delivered_at.is_some()));
}

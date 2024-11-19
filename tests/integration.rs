use std::collections::HashMap;
use std::ops::Deref;

use nervemq::{config::Config, db::namespace::Namespace, db::queue::Queue, service::Service};
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

    TmpService {
        svc: Service::connect_with(Config {
            db_path: Some(path.path().join("nervemq.db").to_string_lossy().to_string()),
        })
        .await
        .unwrap(),
        tmpdir: path,
    }
}

// #[tokio::test]
// async fn test_list_namespace() {
//     let service = setup().await;
//
//     assert_eq!(service.list_namespaces().await.unwrap(), vec![]);
//
//     //create namespace to test
//     service.create_namespace("testing").await.unwrap();
//
//     //checks to see if the list_namespace function contains the newly created namespace
//     assert_eq!(
//         service.list_namespaces().await.unwrap(),
//         vec![Namespace {
//             id: 1,
//             name: "testing".to_owned()
//         }]
//     );
// }

// #[tokio::test]
// async fn test_delete_namespace() {
//     let service = setup().await;
//
//     assert_eq!(service.list_namespaces().await.unwrap(), vec![]);
//
//     //create namespace
//     service.create_namespace("testing").await.unwrap();
//     //checks to see if namespace is created
//     assert_eq!(
//         service.list_namespaces().await.unwrap(),
//         vec![Namespace {
//             id: 1,
//             name: "testing".to_owned()
//         }]
//     );
//     //delete namespace
//     service.delete_namespace("testing").await.unwrap();
//     //checks to see if namespace is deleted
//     assert_eq!(service.list_namespaces().await.unwrap(), vec![]);
// }
//
// #[tokio::test]
// async fn test_delete_queue() {
//     let service = setup().await;
//
//     // Create namespace first
//     service.create_namespace("testing").await.unwrap();
//
//     // Verify no queues exist initially
//     assert_eq!(service.list_queues(Some("testing")).await.unwrap(), vec![]);
//
//     // Create queue
//     service.create_queue("testing", "test-queue").await.unwrap();
//
//     // Verify queue exists
//     assert_eq!(
//         service.list_queues(Some("testing")).await.unwrap(),
//         vec![Queue {
//             id: 1,
//             name: "test-queue".to_owned(),
//             ns: "testing".to_owned()
//         }]
//     );
//
//     // Delete queue
//     service.delete_queue("testing", "test-queue").await.unwrap();
//
//     // Verify queue is deleted
//     assert_eq!(service.list_queues(Some("testing")).await.unwrap(), vec![]);
// }
//
// #[tokio::test]
// async fn test_list_queues() {
//     let service = setup().await;
//
//     // Create namespace
//     service.create_namespace("testing").await.unwrap();
//     // Verify empty initially
//     assert_eq!(service.list_queues(Some("testing")).await.unwrap(), vec![]);
//
//     // Create queue
//     service.create_queue("testing", "test-queue").await.unwrap();
//     // Verify queue is listed
//     assert_eq!(
//         service.list_queues(Some("testing")).await.unwrap(),
//         vec![Queue {
//             id: 1,
//             name: "test-queue".to_owned(),
//             ns: "testing".to_owned()
//         }]
//     );
// }
//
// #[tokio::test]
// async fn test_send_message() {
//     let service = setup().await;
//
//     // Setup namespace and queue
//     service.create_namespace("testing").await.unwrap();
//     service.create_queue("testing", "test-queue").await.unwrap();
//
//     // Send a message with empty HashMap
//     let message = "Hello, World!".as_bytes().to_vec();
//     let kv = HashMap::new();
//     service
//         .send_message("testing", "test-queue", &message, kv)
//         .await
//         .unwrap();
//
//     // Verify message exists
//     let messages = service
//         .list_messages("testing", "test-queue")
//         .await
//         .unwrap();
//     assert_eq!(messages.len(), 1);
//     assert_eq!(messages[0].body, message);
// }
//
// #[tokio::test]
// async fn test_list_messages() {
//     let service = setup().await;
//
//     // Setup namespace and queue
//     service.create_namespace("testing").await.unwrap();
//     service.create_queue("testing", "test-queue").await.unwrap();
//
//     // Send a message
//     let message = "Hello, World!".as_bytes().to_vec();
//     let kv = HashMap::new();
//     service
//         .send_message("testing", "test-queue", &message, kv)
//         .await
//         .unwrap();
//
//     // Verify message exists
//     let messages = service
//         .list_messages("testing", "test-queue")
//         .await
//         .unwrap();
//     assert_eq!(messages.len(), 1);
//     assert_eq!(messages[0].body, message);
// }

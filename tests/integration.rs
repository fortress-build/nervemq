use std::{ops::Deref, path::PathBuf};

use creek::{config::Config, db::namespace::Namespace, service::Service};
use tempfile::{env::temp_dir, TempDir};

struct TmpService {
    svc: Service,
    tmpdir: TempDir,
}

impl Deref for TmpService {
    type Target = Service;

    fn deref(&self) -> &Self::Target {
        &self.svc
    }
}

async fn setup() -> TmpService {
    let path = tempfile::tempdir().unwrap();

    TmpService {
        svc: Service::connect_with(Config {
            db_path: Some(path.path().join("creek.db").to_string_lossy().to_string()),
        })
        .await
        .unwrap(),
        tmpdir: path,
    }
}

#[tokio::test]
async fn test_create_namespace() {
    let service = setup().await;

    assert_eq!(service.list_namespaces().await.unwrap(), vec![]);

    service.create_namespace("testing").await.unwrap();

    assert_eq!(
        service.list_namespaces().await.unwrap(),
        vec![Namespace {
            id: 1,
            name: "testing".to_owned()
        }]
    );
}

use aws_config::{BehaviorVersion, Region};

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    tracing_subscriber::fmt::init();

    let credentials = aws_sdk_sqs::config::Credentials::new(
        "6kkMWFC1nin",
        "FhwbQ682XAe7PxcY7WWkJKGscqdpdknZP",
        None,
        None,
        "Static",
    );

    let config = aws_sdk_sqs::Config::builder()
        .region(Region::new("us-west-1"))
        .credentials_provider(credentials)
        .endpoint_url("http://localhost:8080/sqs")
        .behavior_version(BehaviorVersion::latest())
        .build();

    let sqs = aws_sdk_sqs::Client::from_conf(config);

    let res = sqs.get_queue_url().queue_name("test").send().await;

    tracing::info!("Result: {:?}", res);

    Ok(())
}

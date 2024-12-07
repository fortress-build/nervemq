<div align="center">
  <span>
    <h1>NerveMQ</h1>

  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

  </span>

A lightweight, SQLite-backed message queue with AWS SQS-compatible API, written in Rust.

</div>

## Features

- 🚀 **AWS SQS Compatible API** - Drop-in replacement for applications using AWS SQS
- 💾 **SQLite Backend** - Reliable, embedded storage with ACID guarantees
- 🔒 **Multi-tenant** - Namespace isolation with built-in authentication
- 🔄 **Message Visibility Control** - Configure message processing timeouts
- 📊 **Queue Attributes** - Track message counts, timestamps, and queue settings
- 🏃 **Fast & Efficient** - Written in Rust for optimal performance
- 🎯 **Self-contained** - Self-contained binary with minimal requirements

## Quick Start

TODO

## Configuration

TODO

## Usage Examples

### Using AWS SDK

```rust
use aws_sdk_sqs::{Client, Config};

async fn example() {
    let config = Config::builder()
        .endpoint_url("http://localhost:8080/sqs")
        .build();

    let client = Client::from_conf(config);

    // Send a message
    client.send_message()
        .queue_url("http://localhost:8080/namespace/myqueue")
        .message_body("Hello World!")
        .send()
        .await?;
}
```

### Using HTTP API

```bash
# Send a message
curl -X POST http://localhost:8080/queue/myqueue/messages \
  -H "Content-Type: application/json" \
  -d '{"MessageBody": "Hello World!"}'

# Receive messages
curl http://localhost:8080/queue/myqueue/messages
```

## Why NerveMQ?

- **Simple Deployment**: Single binary, no external dependencies
- **Familiar API**: AWS SQS compatibility means easy migration
- **Reliable Storage**: SQLite provides robust data persistence
- **Cost Effective**: Self-hosted alternative to cloud services
- **Developer Friendly**: Easy to set up for development and testing

## Architecture

NerveMQ uses SQLite as its storage engine, providing:

- ACID compliance
- Reliable message delivery
- Efficient queue operations
- Data durability
- Low maintenance overhead

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- AWS SQS for API inspiration
- SQLite for reliable storage
- Rust community for excellent tools and libraries

---

<div align="center">
Made with ❤️by the Fortress team
</div>

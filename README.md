<div align="center">
  <span>
    <h1>NerveMQ</h1>

[![GitHub License](https://img.shields.io/github/license/fortress-build/nervemq)](https://github.com/fortress-build/nervemq/blob/main/LICENSE)

  </span>

A lightweight, SQLite-backed message queue with AWS SQS-compatible API, written in Rust.

</div>

> [!NOTE]
> This project is still in development and has not been tested in production scenarios.

## Features

- 🚀 **AWS SQS Compatible API** - Drop-in replacement for applications using AWS SQS
- 💾 **SQLite Backend** - Reliable, embedded storage with ACID guarantees
- 🔒 **Multi-tenant** - Namespace isolation with built-in authentication
- 📊 **Queue Attributes** - Track message counts, timestamps, and queue settings
- 🏃 **Fast & Efficient** - Written in Rust for optimal performance
- 🎯 **Self-contained** - Self-contained binary with minimal requirements
- 📱 Admin Interface - Manage queues and tenants via UI or API

## Installation / Quick Start

NerveMQ is intended to be modular and extensible. As such, it can be consumed in two ways: using
the preconfigured binary in `main.rs`, or including `nervemq` as a library and providing the custom
implementations needed for your use-case. We also plan to add more configuration options to the preconfigured
binary so that common use-cases are covered.

For now, you will have to clone the repo from github.

```bash
git clone https://github.com/fortress-build/nervemq
cd nervemq
cargo run --release
```

The server expects a few configuration parameters to be available via
environment variables:

- `NERVEMQ_DB_PATH` (optional; default: `./nervemq.db`)
  Database file path

- `NERVEMQ_DEFAULT_MAX_RETRIES` (optional; default: `10`)
  Default retry limit

- `NERVEMQ_HOST` (optional; default `http://localhost:8080`)
  Server host URL (for UI access)

- `NERVEMQ_ROOT_EMAIL` (optional; default `admin@example.com`)
  Root admin email

- `NERVEMQ_ROOT_PASSWORD` (optional; default `password`)
  Root admin password

The server doesn't have any subcommands or CLI interface. Just run `nervemq` to start.

To use the UI (for now) you must clone the git repo and run the nextjs app manually. We may make a hosted version
available in the future or rework the webapp to be bundled statically and served by the server as well.

```bash
git clone https://github.com/fortress-build/nervemq
cd nervemq
bun run start # node and deno should work here too :)
```

## Usage Examples

NerveMQ's queue APIs are compatible with SQS, so you can you any SQS client.

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

## Admin API

TODO: Document the admin API

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

Copyright 2024 Fetchflow, Inc.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

---

<div align="center">
Made with ❤️by the Fortress team
</div>

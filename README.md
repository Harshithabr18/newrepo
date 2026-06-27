# Project Requirements

## Language
- Rust (Edition 2021)

## Package Manager
- Cargo

## Minimum Rust Version
- rustc 1.70 or later (recommended latest stable)
- cargo 1.70 or later

## Operating Systems
- Windows 10/11
- Ubuntu 22.04+
- macOS 13+

## Required Tools

| Tool | Version |
|------|---------|
| Rust | Edition 2021 |
| Cargo | Latest Stable |
| Git | 2.40+ |
| Docker (Optional) | 24+ |
| Postman (API Testing) | Latest |
| VS Code | Latest |
| Rust Analyzer Extension | Latest |

---

# Rust Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| tokio | 1.x | Async Runtime |
| axum | 0.7 | Web Framework |
| hyper | 1.x | HTTP Server |
| tower | 0.4 | Middleware |
| tower-http | 0.5 | HTTP Utilities |
| serde | 1.x | Serialization |
| serde_json | 1.x | JSON Support |
| toml | 0.8 | Configuration |
| quick-xml | 0.31 | XML Serialization |
| hmac | 0.12 | AWS Signature V4 |
| sha2 | 0.10 | SHA-256 Hashing |
| md-5 | 0.10 | MD5 Hashing |
| hex | 0.4 | Hex Encoding |
| base64 | 0.22 | Base64 Encoding |
| sled | 0.34 | Metadata Database |
| thiserror | 1.x | Error Handling |
| anyhow | 1.x | Error Utilities |
| tracing | 0.1 | Logging |
| tracing-subscriber | 0.3 | Log Subscriber |
| uuid | 1.x | UUID Generation |
| chrono | 0.4 | Date & Time |
| bytes | 1.x | Byte Buffers |
| rand | 0.8 | Random Number Generation |
| futures | 0.3 | Async Utilities |
| async-trait | 0.1 | Async Traits |
| url | 2.x | URL Parsing |
| mime | 0.3 | MIME Types |
| http | 1.x | HTTP Types |
| config | 0.14 | Configuration Management |
| reqwest | 0.12 | HTTP Client |

---

# Development Dependencies

| Dependency | Version |
|------------|---------|
| axum-test | 14 |
| tokio-test | 0.4 |
| tempfile | 3 |

---

# Features

- AWS Signature Version 4 Authentication
- S3-Compatible REST API
- Bucket Management
- Object Upload & Download
- Metadata Management
- Reed-Solomon Erasure Coding
- Async Processing using Tokio
- Structured Logging
- Integration & Unit Testing

---

# Build

```bash
cargo build
```

# Run

```bash
cargo run
```

# Run Tests

```bash
cargo test
```

# Format Code

```bash
cargo fmt
```

# Lint

```bash
cargo clippy
```

# Documentation

```bash
cargo doc --open
```

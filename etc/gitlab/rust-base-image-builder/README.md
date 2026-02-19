# 🦀 Rust Alpine Builder Image

This repository provides a **Docker-based build script** for creating a reusable 
**Rust builder image** based on **Alpine Linux**.  
The image is designed to compile Rust applications efficiently, generate OpenAPI documentation, 
and compress the final executable using **UPX**.

It leverages **cargo-chef** to significantly improve build times by caching dependency 
compilation layers.

---

## 🚀 Features

- Based on `rust:alpine`
- Optimized dependency caching with **cargo-chef**
- Static linking using **musl**
- Binary size reduction using **UPX**
- Suitable for CI/CD pipelines and reproducible builds
- Includes common tools required for OpenAPI generation and Rust builds

---

## ⚙️ Base Image

The builder image is based on:

```
rust:alpine
```

This ensures a lightweight environment while maintaining full Rust toolchain support.

---

## ⚙️ Build Tools and Dependencies

The image installs the following system dependencies required to compile Rust applications 
and produce optimized static binaries:

| Dependency            | Description                                                |
| --------------------- | ---------------------------------------------------------- |
| `upx`                 | Compresses the final Rust executable to reduce binary size |
| `curl`                | Required by build scripts and tooling                      |
| `musl-dev`            | Enables static linking against musl libc                   |
| `openssl`             | OpenSSL runtime library                                    |
| `openssl-dev`         | Headers for compiling crates that depend on OpenSSL        |
| `openssl-libs-static` | Static OpenSSL libraries for fully static binaries         |
| `pkgconfig`           | Helps locate system libraries during build                 |
| `gcc`                 | Required for compiling native dependencies                 |
| `libssl-dev`          | Required lib for compiling static OpenSSL                  |

Installed via:

```dockerfile
RUN apk update && apk add --no-cache upx curl musl-dev openssl\
    openssl-dev pkgconfig gcc openssl-libs-static
```

---

## 🍳 cargo-chef for Faster Builds

This image uses **cargo-chef** to improve Docker build performance by caching Rust dependencies 
separately from application code.

```dockerfile
RUN cargo install --locked cargo-chef
```

This allows dependency layers to be reused across builds as long as `Cargo.toml` 
and `Cargo.lock` remain unchanged.

---

## 🐳 Dockerfile Overview

The Dockerfile follows a multi-stage build strategy:

### 1. Chef Stage – Install Tools and Dependencies

```dockerfile
FROM rust:alpine AS chef
WORKDIR /app
RUN cargo install --locked cargo-chef &&\
    apk update && apk add --no-cache upx curl musl-dev openssl\
    openssl-dev pkgconfig gcc openssl-libs-static
```

---

## 🧩 Use Cases

- CI/CD pipelines for Rust projects
- Building microservices with minimal runtime images
- Projects requiring OpenAPI documentation generation
- Environments where binary size matters

    ### How to Use

    ```dockerfile
    FROM rust-builder:latest AS chef
    WORKDIR /app
    ```

    #### 1. Planner Stage – Dependency Analysis

    ```dockerfile
    FROM chef AS planner
    COPY . .
    RUN cargo chef prepare --recipe-path recipe.json
    ```

    #### 2. Builder Stage – Dependency Compilation

    ```dockerfile
    FROM chef AS builder
    COPY --from=planner /app/recipe.json recipe.json
    RUN cargo chef cook --recipe-path recipe.json --release
    ```

    #### 3. Final Build – Application Binary

    ```dockerfile
    COPY . .
    RUN cargo build --release --target x86_64-unknown-linux-musl
    ```

    ---

    #### 4. Output

    - 📦 A **release-mode Rust binary**
    - Statically linked using `musl`
    - Ready to be compressed with **UPX**
    - Ideal for use in a minimal runtime image (e.g. `scratch` or `alpine`)
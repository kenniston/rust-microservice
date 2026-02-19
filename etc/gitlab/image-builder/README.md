# Builder Base Image Pipeline

This repository provides a **generic GitLab CI pipeline definition
(`builder.yml`)** used as a foundation for building **Docker builder
images**. These images are intended to be reused across multiple
projects as standardized build environments.

A common use case is creating **base images for compiling Rust
applications**, but the same approach can be applied to any language or
toolchain that requires a consistent and reproducible build environment.

## Purpose

The goal of this repository is to:

-   Centralize the definition of **Docker builder image pipelines**
-   Promote **reuse and standardization** across projects
-   Simplify maintenance of build environments
-   Ensure reproducible builds using tagged Docker images

The `builder.yml` file is meant to be **included or referenced** by
other project pipelines that generate Docker images used during the
build stage.

## Repository Structure

    .
    ├── builder.yml
    └── README.md

## How It Works

The pipeline defined in `builder.yml`:

1.  Runs only when a **Git tag** is created.
2.  Uses **Docker-in-Docker (DinD)** to build images.
3.  Builds the Docker image using the current repository's `Dockerfile`.
4.  Tags the image using the Git tag (`CI_COMMIT_TAG`).
5.  Pushes the image to the GitLab Container Registry.

## Usage in Other Projects

This file should be included or adapted by projects that need
standardized builder images, such as Rust compilation images.

## Versioning Strategy

Builder images are versioned using **Git tags**, ensuring immutable and
traceable builds.


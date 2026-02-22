# `/etc` Directory Architecture

This document describes the infrastructure configuration directory of
the Rust REST API project and provides a visual architecture diagram of
how its components interact.

------------------------------------------------------------------------

## Architecture Diagram

```text
                              ┌─────────────────────────────┐
                              │         Developer           │
                              │   Local Development Env     │
                              └──────────────┬──────────────┘
                                             │
                                             ▼
                              ┌─────────────────────────────┐
                              │        docker-compose.yaml  │
                              │   Local Runtime Environment │
                              └──────────────┬──────────────┘
                                             │
            ┌────────────────────────────────┼────────────────────────────────┐
            │                                │                                │
            ▼                                ▼                                ▼
    ┌───────────────────┐         ┌───────────────────┐         ┌────────────────────────┐
    │   Rust REST API   │         │   API Database    │         │     Keycloak Server    │
    │   Application     │         │   Initialization  │         │ Identity & Access Mgmt │
    └─────────┬─────────┘         │   (.api-initdb)   │         │       (.initdb)        │
              │                   └───────────────────┘         └────────────────────────┘
              │
              ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │                                telemetry/                               │
    │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐   │
    │  │   Prometheus│  │     Loki    │  │    Tempo    │  │    Grafana    │   │
    │  │   Metrics   │  │     Logs    │  │   Tracing   │  │ Visualization │   │
    │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘


                                    CI/CD Infrastructure

                              ┌─────────────────────────────┐
                              │ docker-compose-gitlab.yaml  │
                              │        CI/CD Stack          │
                              └──────────────┬──────────────┘
                                             │
            ┌────────────────────────────────┼────────────────────────────────┐
            ▼                                ▼                                ▼
    ┌───────────────────┐         ┌───────────────────┐         ┌──────────────────────┐
    │    GitLab Server  │         │   GitLab Runner   │         │   Container Registry │
    │    Pipelines      │         │   Build Executor  │         │   Image Storage      │
    └───────────────────┘         └───────────────────┘         └──────────────────────┘


                                Local Kubernetes Environment

                              ┌─────────────────────────────┐
                              │             k3d/            │
                              │  Local Kubernetes Cluster   │
                              └─────────────────────────────┘
```

## Directory Structure

    etc
    ├── .api-initdb/
    ├── .initdb/
    ├── gitlab/
    ├── k3d/
    ├── telemetry/
    ├── realm-export.json
    ├── docker-compose.yaml
    └── docker-compose-gitlab.yaml


## Directory Responsibilities

### `.api-initdb/`

Database initialization scripts for the Rust REST API.

Schema creation - Initial data seeding - Database configuration

---

### `.initdb/`

Keycloak database initialization resources.

Realm configuration - Identity provider setup - Roles and clients initialization

---

### `gitlab/`

CI/CD infrastructure configuration.

Runner configuration - Pipeline support files - Environment preparation

---

### `k3d/`

Local Kubernetes cluster configuration using K3D.

Cluster setup - Networking configuration - Local deployment environment

---

### `telemetry/`

Observability stack configuration.

Prometheus → metrics collection - Loki → log aggregation -
Tempo → distributed tracing - Grafana → visualization and dashboards

---

### `realm-export.json`

**The `realm-export.json` file contains essential Keycloak configuration for the 
development environment, including clients, secrets, and users. This data is 
used for API development and integration testing.**

---

## Docker Compose Environments

### `docker-compose.yaml`

Local runtime environment for the Rust REST API and dependencies.

### `docker-compose-gitlab.yaml`

CI/CD infrastructure environment including GitLab and related services.

---

## Summary

The `/etc` directory centralizes infrastructure configuration,
environment setup, and observability tooling, ensuring reproducible
local development, consistent CI/CD execution, and standardized
monitoring across the project.

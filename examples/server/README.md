# Rust Server API

![rust](https://badgen.net/badge/Rust%20Edition/2024/red?scale=1.2)
![rust](https://badgen.net/badge/Rust/1.91.1/blue?scale=1.2)
![cargo](https://badgen.net/badge/Cargo/1.91.1/gray?scale=1.2)
![spring-boot](https://badgen.net/badge/Version/0.1.2/green?scale=1.2)

## About the Project

This microservice was developed in Rust with a strong focus on high performance, security, 
and scalability.
It exposes a REST API responsible for data querying, creation, update, and deletion operations,
structured according to the MVC (Model–View–Controller) architectural pattern, promoting a 
clear separation of responsibilities and facilitating code maintenance and evolution.

The **Controller** layer of each module is responsible for receiving and handling HTTP requests,
performing initial validations, and routing the flow to the appropriate components. The 
**Model** layer—represented here by the objects in the **Service** module of each feature—is 
responsible for business rules, while data access logic is represented by the objects in the 
**Repository** module of each feature.

The microservice leverages modern libraries from the Rust ecosystem, such as actix-web 
for HTTP route management, serde for data serialization and deserialization, and SeaORM 
for database communication.
The entire flow follows Rust’s recommended practices for concurrency and memory safety, 
resulting in a robust, efficient service ready for production environments.

---
## 🚀 Technologies Used

- **Rust**
- **Kubernetes**
- **TestContainers** (integration tests)
- **Serde**, **Tokio**, **Actix**, SeaORM, and other crates

---
## 📦 API Features

- Query users stored in the database;
- Versioned endpoints;
- Filters by name and document;
- Structured logging;
- Configuration via environment variables.

---
## ☸️ Kubernetes Deployment

The server can runs on a Kubernetes cluster containing:

- Deployment
- ConfigMap
- Secret
- Service
- Ingress

---
## 🧪 Tests

### ✔️ Unit Tests
Run with:

```
cargo test
```

### ✔️ Integration Tests with TestContainers

The tests start real containers to validate the API behavior in an isolated environment:

```rust
use testcontainers::clients::Cli;

#[test]
fn integration_test() {
    let docker = Cli::default();
    let container = docker.run(MyContainer::default());
}
```

---
## 🛠️ How to Run Locally

>IMPORTANT: Proceed with the steps described in [Development Environment](#development-environment)
> to fully prepare the development environment for this project.

```
cargo run -- --config-file="./assets/your-custom-config.yaml" run
```

---
## 📁 Project Structure

```
root
├── assets/                                 # Static files, mock data, and test resources
│     └── tests/                            # Assets specifically used for integration or unit tests
│
├── src/                                    # Main Rust source code
│    ├── dto/                               # Data Transfer Objects for API input/output
│    ├── entity/                            # Project Database Entity
│    ├── module/                            # Application feature modules
│    │     ├── user/                        # "User" domain module
│    │     │    ├── user_controller.rs      # HTTP handlers and route definitions
│    │     │    ├── user_repository.rs      # Data access layer (BigQuery)
│    │     └──  └── user_service.rs         # Business logic and service layer
│    │
│    └── main.rs                            # Application entry point
│
└── tests/                                  # Integration tests executed with cargo test
```

---
## Accessing the local API

To access the API, open the following address in your browser after prepare the
development environment:

[http://localhost:8080/swagger-ui.html](http://localhost:8080/swagger-ui.html)

---
## 🛠️ Development Environment

Proceed with the steps bellow to fully prepare the development enviroment:

### 1. Starts the CI/CD containers

&emsp;&emsp; 1.1 - Execute the steps described in [`GitLab Environment`](etc/gitlab/README.md)

&emsp;&emsp; 1.2 - Starts the CI/CD Containers defined in 
                   [/etc/docker-compose-gitlab.yaml`](/etc/docker-compose-gitlab.yaml)

### 2. Starts the Application and Monitoring containers

&emsp;&emsp; 2.1 - Execute the steps described in [`Application Environment`](etc/README.md)

&emsp;&emsp; 2.2 - Starts the Application Containers defined in
                   [/etc/docker-compose.yaml`](/etc/docker-compose.yaml)

### 3. Starts the local Kubernetes Cluster (K3D)

&emsp;&emsp; Execute the steps described in [`K3D Environment`](etc/k3d/README.md)


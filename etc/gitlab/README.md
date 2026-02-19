# CI/CD Configuration

This directory contains all configuration files required to run **GitLab**, **GitLab Runner**,
and a **custom SSL setup using a self-signed Certificate Authority (CA)**.  
All files in this directory are consumed by the `docker-compose-gitlab.yaml` file located in
the `/etc` directory of the project.

The default configuration starts the GitLab server without SSL (recommended for simplicity).

> Approximately **6 GB of RAM is required** to run the full CI/CD environment (GitLab, Runner,
> Nexus Repository, and SonarQube).
>
> ***IMPORTANT: To avoid Docker Hub pull rate limits, all base images used in this project 
> must be published to the local Nexus registry as shown below:***
>
> ![Docker Images - Local Registry](/images/docker-images-local-registry.png)
> 
> To push a new image to the local registry, run the following commands:
>
> 1. Pull the desired image:
> ```sh
> docker pull alpine:latest
> ```
> 2. Tag the image for the local registry:
> ```sh
> docker tag alpine:latest nexus:5005/alpine:lates
> ```
> 3. Push the image to the local repository:
> ```sh
> docker push nexus:5005/alpine:latest
> ```
> &nbsp;


## Directory Structure

```text
gitlab
├── gitlab-runner             # Base configuration for the GitLab Runner.
│   └── runner
│       └── config
│           └── config.toml
│
├── image-builder             # Template used for building images.
│   ├── builder.yml
│   └── README.md
│
├── rust-base-image-builder   # Provides the required toolchain for compiling Rust applications.
│   ├── .gitlab-ci.yml
│   ├── Dockerfile
│   └── README.md
│
└── ssl                       # Includes SSL certificates for GitLab instances running behind HTTPS.
    ├── ca.key
    ├── ca.pem
    ├── ca.srl
    ├── extfile.cnf
    ├── gitlab.server.crt
    ├── gitlab.server.csr
    └── gitlab.server.key
```

## GitLab Runner Configuration

The `gitlab-runner` directory contains the configuration required for registering and running
GitLab Runners.

### `config.toml`

- Defines runner settings such as:
  - Executor type (e.g. `docker`)
  - Docker image used for jobs
  - Volumes and cache configuration
  - Environment variables
- This file is mounted directly into the GitLab Runner container by `docker-compose-gitlab.yaml`.

Path:
```text
gitlab/gitlab-runner/runner/config/config.toml
```

## SSL Configuration (Self-Signed CA)

The `ssl` directory contains all files required to create and use a self-signed SSL certificate
trusted by a custom Certificate Authority.

### Files Description

- **ca.key**  
  Private key of the self-signed Certificate Authority.

- **ca.pem**  
  Public certificate of the Certificate Authority.

- **ca.srl**  
  Serial number file automatically generated when signing certificates.

- **extfile.cnf**  
  OpenSSL configuration file containing certificate extensions (SAN, key usage, etc.).

- **gitlab.server.key**  
  Private key for the GitLab server.

- **gitlab.server.csr**  
  Certificate Signing Request (CSR) for the GitLab server.

- **gitlab.server.crt**  
  SSL certificate for the GitLab server, signed by the custom CA.

## Certificate Generation Workflow

Below is the recommended process to generate the certificates contained in this directory:

1. **Create the CA private key**
   ```bash
   openssl genrsa -out ca.key 4096
   ```

2. **Create the CA certificate**
   ```bash
   openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 -out ca.pem
   ```

3. **Generate the GitLab server private key**
   ```bash
   openssl genrsa -out gitlab.server.key 4096
   ```

4. **Create the server CSR**
   ```bash
   openssl req -new -key gitlab.server.key -out gitlab.server.csr
   ```

5. **Sign the server certificate with the CA**
   ```bash
   openssl x509 -req -in gitlab.server.csr -CA ca.pem -CAkey ca.key -CAcreateserial \
           -out gitlab.server.crt -days 825 -sha256 -extfile extfile.cnf
   ```

## Usage with Docker Compose

- The `docker-compose-gitlab.yaml` file located in `/etc` mounts:
  - `gitlab/gitlab-runner` into the GitLab Runner container
  - `gitlab/ssl` into the GitLab container for HTTPS support
- The generated CA certificate (`ca.pem`) can be imported into:
  - Host operating system trust store
  - Docker daemon trust store
  - Browsers and CI environments

This ensures secure HTTPS communication with GitLab using a trusted internal CA.


## Nexus Repository for Docker Images

This project also uses **Sonatype Nexus Repository Manager 3** as a private Docker image registry.  
The Nexus service is deployed using Docker Compose and is connected to the same Docker network 
as GitLab.

### Nexus Docker Compose Service

Below is the Docker Compose service definition used to install Nexus:

```yaml
nexus:
  image: sonatype/nexus3
  container_name: nexus
  restart: always
  ports:
    - "8081:8081"
    - "5005:5005"
  networks:
    - gitlab
  volumes:
    - '~/gitlab/nexus:/nexus-data'
```

- **Port 8081**: Nexus web interface  
- **Port 5005**: Docker Hosted repository endpoint  
- **Volume `/nexus-data`**: Persistent storage for Nexus configuration, repositories, and credentials

### Initial Admin Password

After the first startup, Nexus generates a temporary admin password.  
This password is stored inside the mounted volume and can be retrieved with:

```sh
cat /nexus-data/admin.password
```

Use this password to log in to the Nexus web interface at:

```
http://<host>:8081
```

You will be prompted to set a new admin password during the initial setup.

### Configuring a Docker Hosted Repository

To configure Nexus as a Docker image registry, follow the steps below:

1. Log in to the Nexus web interface as **admin**
2. Navigate to:
   **Settings → Repositories → Create repository**
3. Select:
   **Docker (hosted)**
4. Configure the repository:
   - **Name**: `docker-hosted`
   - **HTTP Port**: `5005`
   - **Enable Docker V1 API**: Disabled (unless legacy support is required)
   - **Blob store**: `default`
   - **Deployment policy**: `Allow redeploy` (recommended for CI usage)
5. Save the repository

Once created, the Docker registry will be available at:

```text
<host>:5005
```

### Using Nexus with Docker

To authenticate Docker with Nexus:

```sh
docker login <host>:5005
```

Use your Nexus username and password.

To tag and push images:

```sh
docker tag my-image:latest <host>:5005/my-image:latest
docker push <host>:5005/my-image:latest
```

### Integration with GitLab CI

- GitLab CI pipelines can push images to Nexus using the Docker executor
- If Nexus uses HTTP or a self-signed certificate, configure Docker with:
  - Insecure registry settings, or
  - Trusted CA certificates (recommended)

This setup allows GitLab CI jobs to build, tag, and store Docker images securely in a private
registry managed by Nexus.

### Using the registry with Docker in Docker (DinD)

To use Nexus as a private repository in GitLab pipelines, it is necessary to configure the DinD
service with an insecure registry, as shown below:

```yaml
stages:
  - build

services:
  - name: docker:29.2.0-dind
    alias: docker
    command: [
      "--tls=false",
      "--insecure-registry=nexus:5005"  # <=========== Important Configuration
    ]

docker build and push:
  stage: build
  rules:
    - if: $CI_COMMIT_TAG
  image: docker:29.2.0
  tags:
    - docker
  before_script:
    - until docker info; do sleep 1; done;
  script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker build --no-cache --pull -t $CI_BUILDER_BASE_URL/$CI_PROJECT_NAME:$CI_COMMIT_TAG .
    - docker push $CI_BUILDER_BASE_URL/$CI_PROJECT_NAME:$CI_COMMIT_TAG
```

The `GitLab Runner` **must** be connected to the **same GitLab network**, as shown below:

```toml
concurrent = 1
check_interval = 0
shutdown_timeout = 0

[session_server]
session_timeout = 1800

[[runners]]
name = "linux_docker"
url = "https://gitlab.server/"
id = 1
token = "glrt-sGGsr12ZIwG0IcQWeyqK7m86MQp0OjEKdToxCw.01.120tby21a"
token_obtained_at = 2026-01-30T15:26:36Z
token_expires_at = 2027-01-30T15:26:36Z
executor = "docker"
environment = ["FF_NETWORK_PER_BUILD=1"]
tls-ca-file = "/etc/gitlab-runner/certs/gitlab.server.crt"
tls-skip-verify = true

# To authenticate and connect to the Nexus registry, the runner requires the 
# DOCKER_AUTH_CONFIG setting.
environment = [
  "DOCKER_HOST=tcp://docker:2375",
  "DOCKER_CERT_PATH=",
  "DOCKER_TLS_CERTDIR=",
  "DOCKER_DRIVER=overlay2",
  "DOCKER_AUTH_CONFIG={\"auths\":{\"nexus:5005\":{\"auth\":\"Z2l0bGFiOjEyMzQ1Ng==\"}}}"
]

[runners.cache]
MaxUploadedArchiveSize = 0

[runners.cache.s3]
[runners.cache.gcs]
[runners.cache.azure]

[runners.docker]
tls_verify = false
image = "docker:29.2.0"
privileged = true
disable_entrypoint_overwrite = false
oom_kill_disable = false
disable_cache = false
volumes = [
    "/cache"
]
shm_size = 0
network_mtu = 0
extra_hosts = ["gitlab.server:172.17.0.1"]  # <==== Required configuration for communication between the GitLab Runner and GitLab.
network_mode = "rust-gitlab-ci_gitlab"  # <==== MUST BE ON THE SAME NETWORK AS THE GITLAB CONTAINER
command = [
    "--tls=false",
    "--insecure-registry=nexus:5005"
]
```

> The GitLab Runner uses the host’s Docker daemon to pull images. Therefore, to use the 
> nexus hostname, it must be defined in the `/etc/hosts` file on the host machine, assuming 
> the operating system is Linux.
>
> `/etc/hosts`
>
> ```txt
> 127.0.0.1   localhost
> 127.0.0.1   gitlab.server
> 127.0.0.1   nexus
> 127.0.0.1   sonar
> ```

## GitLab Group and Projects Configuration

The recommended GitLab group and project structure for this repository is as follows:

### 1. Create a grup named `Rust` in GitLab:

    ![Docker Images - Local Registry](/images/gitlab-rust-group.png)

<br/>

### 2. Configure the environment variables for the `Rust` group as follows:

    | Variable             | Value                                           | Environments  |
    | -------------------- | ----------------------------------------------- | ------------- |
    | BINARIES_PATH        | `.binaries`                                     | All (default) |
    | BINARY_COMPRESSION   | `<true> or <false>`                             | All (default) |
    | CI_BUILDER_BASE_URL  | `$CI_REGISTRY/ci-cd/docker-base-images/builder` | All (default) |
    | CI_REGISTRY          | `nexus:5005`                                    | All (default) |
    | CI_REGISTRY_PASSWORD | `123456`                                        | All (default) |
    | CI_REGISTRY_USER     | `gitlab`                                        | All (default) |
    | CI_TEMPLATES_HOME    | `rust/ci-cd/pipeline-templates`                 | All (default) |
    | KUBE_CONFIG          | `<GET THE KUBE CONFIG FROM K3D>`                | All (default) |
    | SONAR_HOST_URL       | `http://sonarqube:9000`                         | All (default) |
    | SONAR_TOKEN          | `sqp_4b2038020eeca08ca096f556beec8ffe423cdcb6`  | All (default) |
    | NAMESPACE            | `rust-dev`                                      | All (default) |
    | INGRESS_URL          | `localhost`                                     | All (default) |
    

<br/>

### 3. Create the following structure within the `Rust` group:

    ![Rust Group](/images/rust-group-structure.png)

<br/>

### 4. Create the Rust Builder Image project as shown below:
   
    ![Rust Builder Image Project](/images/rust-builder-image-project.png)

    The project files are located at `/etc/gitlab/rust-base-image-builder`

<br/>

### 5. Create the Builder Template project as shown below:
    
    ![Rust Builder Image Template Project](/images/rust-builder-template-project.png)

    The project files are located at `/etc/gitlab/image-builder`

<br/>

### 6. Create the GitLab Project for the Server API as follow:

    ![Rust Server API Project](/images/server-api-gitlab-project.png)

    **This GitLab project consists of all files contained in this repository.**


## Testing the CI/CD Environment

To validate the CI/CD environment configuration, trigger the build, test, and deploy stages.
To proceed, create a new tag in the project repository on the local GitLab instance.
This tag will trigger a new pipeline in the repository, as shown below:

### **Step 1**: Create a new project Tag:

![Rust Server API Project New Tag](/images/project-new-tag.png)

### **Step 2**: Creating the tag triggers a new pipeline in the repository, as shown below:

![Rust Server API Project Pipeline](/images/project-pipeline.png)

### **Step 3**: The pipeline build, test and deploy the project:

![Rust Server API Project Pipeline Stages](/images/project-pipeline-stages.png)

<br/>

## Project Pipeline

The project pipeline has 3 stages: Build, Test and Deploy. Bellow, are the tasks
executed in each stage:

### Build Task
 
  - Compile the project using Cargo and store the resulting executable in the .binaries directory.
  - If the `BINARY_COMPRESSION` environment variable is set to `true`, the executable is compressed.
  - At the end of this stage, a minimal scratch image is created containing only the final 
    server executable.

  <br/>

> IMPORTANT: Compressing the executable can significantly reduce the program size; however, 
> it may greatly increase the build time.
  
  <br/>

  ![Rust Server API Project Build Stage](/images/project-pipeline-build-stage.png)

  <br/>

  ![Rust Server API Project Nexus](/images/project-pipeline-build-nexus.png)

### Test Task

  - Run the project’s Unit Tests and Integration Tests using cargo llvm-cov.
  - At the end of this stage, an .lcov report is generated and sent to SonarQube, as shown below:

  ![Rust Server API Project Test Stage](/images/project-pipeline-test-stage.png)

  <br/>
  
  ![Rust Server API Project Sonar](/images/project-pipeline-test-sonar.png)


### Deploy Task

This stage deploys the application to a local Kubernetes cluster running on K3D using kubectl
commands executed by the GitLab CI pipeline. During deployment, the pipeline applies a set 
of Kubernetes manifests that provision and configure the runtime environment inside the 
target namespace:

- ConfigMap – Provides the base64-encoded application configuration (B64_CONFIG_FILE).
- Secret – Stores sensitive environment data required by the application.
- Deployment – Creates and manages the rust-server-api container, including health checks, 
  environment configuration, and image version defined by ${IMAGE_TAG}.
- Services
  - API service (port 8080)
  - Health service (port 7188)
- Middleware – Defines request path prefix stripping for /api and /actuator.
- Ingress – Exposes the application externally through the default Traefik ingress controller
  bundled with K3D.

The deployment ensures the application is available internally via ClusterIP services and 
externally through the configured ingress host, enabling both API access and health 
monitoring endpoints.

#### Server Pods
![Rust Server API Project Pods](/images/project-pipeline-deploy-pods.png)

#### Server Services
![Rust Server API Project Services](/images/project-pipeline-deploy-services.png)

#### Server Ingress
![Rust Server API Project Ingress](/images/project-pipeline-deploy-ingress.png)

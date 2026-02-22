# K3D and Kubernetes Configuration

This guide describes how to install and configure a local Kubernetes
cluster using K3D, configure a Docker image registry, and install
cert-manager using Helm.

## Local Kubernetes Cluster with K3D (Helm-based cert-manager Installation)

The setup integrates with the existing project infrastructure,
including:

- Registry configuration (Nexus): **./etc/k3d/registries.yaml**
- Docker network: **rust-gitlab-ci_gitlab**

### Prerequisites

- Linux environment
- Docker installed and running
- kubectl installed
- Helm installed

Verify that the required tools are installed by running the following commands:

***Docker***
```sh
docker info
```

***Kubectl***
```sh
kubectl version --client
```

***Helm***
```sh
helm version
```
---

### Install K3D (Linux)

Install using the official installation script:

```sh
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
```

Verify installation:

```sh
k3d version
```

---

### Registry Configuration

The cluster uses a local Docker registry (Nexus) defined in `./etc/k3d/registries.yaml`

Example structure:

``` yaml
mirrors:
  "nexus:5005":
    endpoint:
      - "http://nexus:5005"

configs:
  "nexus:5005":
    auth:
      username: gitlab
      password: 123456
    tls:
      insecure_skip_verify: true
```
The above configuration enables the Kubernetes cluster to retrieve container images from
a local registry without TLS validation.


> The "DNS" for **http://nexus:5005** can be defined in the `/etc/hosts` file on the 
> host machine, assuming the operating system is Linux.
> 
> `/etc/hosts`
>
> ```txt
> 127.0.0.1   localhost
> 127.0.0.1   gitlab.server
> 127.0.0.1   nexus
> 127.0.0.1   sonar
> ```
> <br/>


---

### Create the K3D Cluster

Create the cluster attached to the CI/CD Docker network:

```sh
k3d cluster create rust-cluster \
  -p "8080:80@loadbalancer" \
  -p "7188:80@loadbalancer" \
  --network rust-gitlab-ci_gitlab \
  --registry-config ./etc/k3d/registries.yaml
```

> IMPORTANT: The cluster must be connected to the same network as the Nexus Registry.


Verify cluster:

***Cluster Info***
```sh
kubectl cluster-info
```

***Cluster Nodes***
```sh
kubectl get nodes
```
---

### Create the Project Kubernetes Namespace

To work properly, the project must be deployed to a specific namespace in the cluster.
The following command creates the `rust-dev` namespace in the cluster:

```sh
kubectl create namespace rust-dev
```

---

### Install cert-manager using Helm (Optional)

Cert Manager automates TLS certificate provisioning and lifecycle management inside Kubernetes.


> **IMPORTANT: If you don’t have a public DNS, use `localhost` for the `API Ingress configuration`
> to prevent browsers from blocking requests when using self-signed certificates. This section 
> can be skipped when using `localhost` for Ingress.**

#### **Step 1 - Add Jetstack Helm Repository**

```sh
helm repo add jetstack https://charts.jetstack.io --force-update
```

---

#### **Step 2 - Install cert-manager via Helm**

```sh
helm upgrade cert-manager jetstack/cert-manager  \
    --install \
    --namespace cert-manager \
    --create-namespace \
    --version=1.17.0 \
    --set crds.enabled=true
```  

This installs:

-   cert-manager controller
-   webhook
-   CA injector

---

#### **Step 3 - Verify Installation**

```sh
kubectl get pods -n cert-manager
```

All pods must be in Running state.

---

#### **Step 4 - Create the Custer Issuer (HTTP)**

```sh
kubectl apply -f ./etc/k3d/letsencrypt-dev.yaml
```
---

### Why cert-manager is Required

cert-manager provides:

-   Automated TLS certificate provisioning
-   Internal certificate authorities
-   Secure ingress configuration
-   Certificate renewal automation

It enables HTTPS support for local services and Kubernetes ingress
resources.

---

### Summary

***Cluster Lifecycle Commands***

**Create cluster**:

```sh
k3d cluster create rust-cluster \
  -p "8080:80@loadbalancer" \
  -p "7188:80@loadbalancer" \
  --network rust-gitlab-ci_gitlab \
  --registry-config ./etc/k3d/registries.yaml
```

**Delete cluster**:

```sh
k3d cluster delete rust-cluster
```

**List clusters**:

```sh
k3d cluster list
```

This setup provides:

-   Local Kubernetes cluster using K3D
-   Integration with CI/CD Docker network
-   Local image registry support (Nexus)
-   TLS certificate automation using cert-manager via Helm

This environment is intended for local development, testing, and CI/CD
integration.

# ⚙️ GitLab SSL Certificate Authority (CA) and Certificate

This folder contains a step-by-step guide and the necessary commands to create a private Certificate Authority (CA) and generate certificates signed by this CA. These certificates are intended to be used with a GitLab server, enabling secure TLS/SSL communication.

The instructions cover the creation of the CA, the generation of server certificates, and the signing process using the created CA.

> The default password for the `ca.key` file in this folder is `123456`.

## 1. Create the Root CA 
Generate the private key and certificate for your own Certificate Authority. 
bash

### Generate CA private key
```sh
openssl genrsa -out ca.key 2048
```

### Generate CA certificate
```sh
openssl req -x509 -new -nodes -key myCA.key -sha256 -days 3650 -out ca.pem \
  -subj "/CN=Rust Local CA"
```

## 2. Create the Server Key and CSR 
Generate a private key and a Certificate Signing Request (CSR) for your services. 
bash

### Generate server private key
```sh
openssl genrsa -out server.key 2048
```

### Create CSR
```sh
openssl req -new -key server.key -out server.csr \
  -subj "/CN=gitlab.server"
```

## 3. Create the Extension Configuration File 
Create a file named extfile.cnf to define the DNS names for the certificate, allowing it to work for both gitlab and registry. 
ini

```cnf
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = gitlab.server
DNS.2 = registry.gitlab.server
DNS.3 = gitlab
IP.1 = 127.0.0.1
IP.2 = 172.17.0.1
IP.3 = 172.18.0.2
```

## 4. Sign the Certificate 
Use the CA to sign the CSR, creating the final certificate. 

```sh
openssl x509 -req -in server.csr -CA myCA.pem -CAkey myCA.key \
  -CAcreateserial -out server.crt -days 365 -sha256 -extfile extfile.cnf
```

## 5. Deployment

**ca.pem**: create a volume to be mounted by both the gitlab-ce and gitlab-runner containers.

Example:

```yaml
gitlab:
    image: gitlab/gitlab-ce
    container_name: gitlab
    restart: always
    hostname: 'gitlab.server'
    environment:
    GITLAB_OMNIBUS_CONFIG: |
        external_url "https://gitlab.server"
        
        ## Nginx Configuration
        nginx['enable'] = true
        nginx['redirect_http_to_https'] = false
        nginx['redirect_http_to_https_port'] = 80
        nginx['ssl_certificate'] = "/etc/gitlab/ssl/gitlab.server.crt"
        nginx['ssl_certificate_key'] = "/etc/gitlab/ssl/gitlab.server.key"
volumes:
    - '/home/server/gitlab/ssl:/etc/gitlab/ssl:ro'

gitlab-runner:
    image: gitlab/gitlab-runner
    container_name: gitlab-runner
    restart: unless-stopped
    links:
    - gitlab
    volumes:
    - '/home/server/gitlab/ssl:/etc/gitlab-runner/certs:ro'
```

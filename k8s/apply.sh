#!/bin/sh

# Before executing this script, ensure that the namespace already
# exists in the K3D cluster.
printf "ℹ️ Configure the k8s namespace...\n"
export NAMESPACE="rust-dev"

# Set the image tag to the most recently published version in Nexus.
printf "ℹ️ Configure the image tag...\n"
export IMAGE_TAG="nexus:5005/rust-server-api:122"

# Set the ingress url
printf "ℹ️ Configure the API ingress url...\n"
export INGRESS_URL="localhost"

printf "ℹ️ Configure the Health ingress url...\n"
export HEALTH_INGRESS_URL="localhost"

printf "ℹ️ Apply the k8s resources...\n"
envsubst < ./k8s/deployment.yaml | kubectl apply -f -

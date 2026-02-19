#!/bin/sh

# Script Name: execute-tests.sh
# Author: Kenniston Arraes Bonfim
# Date: February 4, 2026
# Version: 1.0
# Description: This script executes the tests for the project.
#
set -e

deploy() {
    echo "ℹ️ Deploying application to Kubernetes..."

    echo "ℹ️ Configuring Kubernetes client..."
    if [ ! -e "$HOME/.kube" ]; then
        mkdir -p "$HOME/.kube"
    fi
    printf "%s" "$KUBE_CONFIG" | base64 -d > "$HOME/.kube/config"

    echo "ℹ️ Creating namespace '$NAMESPACE'..."
    if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        echo "Namespace '$NAMESPACE' already exists, skipping creation."
    else
        kubectl create namespace "$NAMESPACE"
    fi

    echo "ℹ️ Deploying application to Kubernetes..."
    envsubst < ./k8s/deployment.yaml | kubectl apply -f -

    echo "✅ Application deployed successfully!"
}

deploy

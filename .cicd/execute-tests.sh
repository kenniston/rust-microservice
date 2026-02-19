#!/bin/sh

# Script Name: execute-tests.sh
# Author: Kenniston Arraes Bonfim
# Date: February 4, 2026
# Version: 1.0
# Description: This script executes the tests for the project.
#
set -e

validation() {
    echo "ℹ️ Validating Cargo.toml file..."
    cargo check --quiet
    echo "✅ Cargo.toml file validated successfully!"
}

test() {
    echo "ℹ️ Running Tests..."
    RUST_TEST_THREADS=1 cargo llvm-cov --lcov --output-path target/lcov.info test

    echo "ℹ️ Copying lcov file..."
    mkdir -p .reports
    cp target/lcov.info .reports/lcov.info

    echo "ℹ️ Running Sonar Scanner..."
    sonar-scanner

    echo "✅ Tests completed successfully!"
}

test

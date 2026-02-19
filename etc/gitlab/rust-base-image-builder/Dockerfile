# (1) installing cargo-chef & build deps
FROM nexus:5005/rust:alpine AS chef

RUN apk update \
    && apk add --no-cache upx curl musl-dev openssl \
    openssl-dev pkgconfig gcc openssl-libs-static envsubst \
    perl make cmake unzip openjdk21 build-base kubectl \
    && cargo install --locked cargo-chef \
    && cargo install --locked cargo-llvm-cov \
    && cargo install --locked cargo-nextest \
    && rustup component add clippy-preview \
    && rustup component add llvm-tools-preview --toolchain 1.93.0-x86_64-unknown-linux-musl

# Set the SonarScanner home directory
ENV SONAR_SCANNER_HOME=/opt/sonar-scanner

# Download and extract the SonarScanner CLI
RUN curl -sSLo /tmp/sonar-scanner.zip https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-8.0.1.6346.zip \
    && unzip -q /tmp/sonar-scanner.zip -d /opt \
    && mv /opt/sonar-scanner-* $SONAR_SCANNER_HOME \
    && rm /tmp/sonar-scanner.zip

# Add the SonarScanner bin directory to the PATH
ENV PATH=$SONAR_SCANNER_HOME/bin:$PATH

# First stage - Rust build
FROM rust AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-gnu

# Second stage - Ubuntu setup
FROM ubuntu:22.04

# Install Node.js
RUN apt-get update && apt-get install -y curl
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash -
RUN apt-get install -y nodejs

# Install Playwright
RUN npm install -g playwright
RUN playwright install-deps

# Install Chromium using Playwright
RUN npx playwright install chromium webkit firefox

# The rest of your existing Dockerfile
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN apt-get update && \
    apt-get install -y \
    libnss3 \
    xvfb \
    xdotool \
    libnss3-tools \
    libnss3-dev \
    libnspr4 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libdbus-1-3 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libasound2 \
    libatspi2.0-0 \
    libxshmfence1 \
    libpango1.0-0 \
    libcairo2 \
    wget && \
    wget https://mirror.combahton.net/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1-1ubuntu2.1~18.04.21_amd64.deb && \
    dpkg -i libssl1.1_1.1.1-1ubuntu2.1~18.04.21_amd64.deb && \
    rm -f libssl1.1_1.1.1-1ubuntu2.1~18.04.21_amd64.deb && \
    apt-get install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    echo "source $HOME/.cargo/env" >> ~/.bashrc && \
    /bin/bash -c "source ~/.bashrc" && \
    /root/.cargo/bin/rustup install stable && \
    /root/.cargo/bin/rustup default stable && \
    /root/.cargo/bin/rustup update && \
    apt-get install -y cargo

# Copy the app into the Docker image
COPY . /app

# Set the working directory to the app
WORKDIR /app

# Build the app
RUN cargo build --release

# Run the app
CMD ["./target/release/web_crawler"]

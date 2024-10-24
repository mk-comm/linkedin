# First stage - Rust build
FROM rust AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-gnu

# Second stage - Ubuntu setup
FROM ubuntu:22.04
# Step 1: Update package lists and install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    gnupg \
    && rm -rf /var/lib/apt/lists/*
# Add Google's official GPG key and set up the stable repository
RUN curl -fsSL https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list

# Install Google Chrome
RUN apt-get update && apt-get install -y \
    google-chrome-stable \
    --no-install-recommends

# Install other required dependencies
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
    wget

# Install OpenSSL 1.1 
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1-1ubuntu2.1~18.04.23_amd64.deb && \
    dpkg -i libssl1.1_1.1.1-1ubuntu2.1~18.04.23_amd64.deb && \
    rm -f libssl1.1_1.1.1-1ubuntu2.1~18.04.23_amd64.deb


# Copy the compiled app from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-gnu/release/web_crawler /app/web_crawler

# Set the working directory to the app
WORKDIR /app

# Add the xvfb.sh script to the image
COPY xvfb.sh /home/apps/xvfb.sh

# Make the script executable
RUN chmod +x /home/apps/xvfb.sh

# Set the entrypoint to run the xvfb.sh script
ENTRYPOINT ["bash", "/home/apps/xvfb.sh"]

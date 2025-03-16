FROM rust:1.79.0-bookworm

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y --no-install-recommends --option=Dpkg::Options::=--force-confdef \
    git zip build-essential curl wget rust-all pkg-config libssl-dev python3-protobuf protobuf-compiler ca-certificates openssl net-tools iproute2 can-utils kmod && \
    apt clean

RUN RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install cargo-tarpaulin cargo-deb

# Compile demoDevice for testing
RUN mkdir /root/.ssh && ssh-keyscan github.com >> /root/.ssh/known_hosts && \
    cd root && git clone https://github.com/CANopenNode/CANopenDemo.git && \
    cd CANopenDemo && \
    git reset --hard b2a6b53 && \
    git submodule update --recursive --init CANopenLinux && \
    cd demo && make
FROM lukemathwalker/cargo-chef:latest as chef

WORKDIR /app

FROM chef AS planner

RUN mkdir -p orm/src
RUN mkdir -p server/src
RUN echo 'fn main() { panic!("Dummy Image Called!")}' > ./orm/src/lib.rs
RUN echo 'fn main() { panic!("Dummy Image Called!")}' > ./server/src/main.rs

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ["./orm/Cargo.toml", "./orm/"]
COPY ["./server/Cargo.toml", "./server/"]

RUN cargo chef prepare

FROM chef AS builder

RUN apt-get update
RUN apt-get install -y curl build-essential ca-certificates tzdata net-tools pkg-config libssl-dev openssl

# Install libtorch. Two things must line up here:
#  * version: must match what `torch-sys`/`tch` (pulled in via rust-bert) expects
#    - tch 0.17.x wants PyTorch 2.4.0, otherwise the build dies with
#    "this tch version expects PyTorch 2.4.0, got 2.1.0"
#  * variant: CPU-only. This host has no NVIDIA GPU/driver, and the app has always
#    run on CPU (rust-bert falls back via Device::cuda_if_available). The cu121
#    build is 2.4GB and useless here; the CPU build is ~190MB.
RUN curl -L https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.4.0%2Bcpu.zip -o libtorch.zip
RUN unzip libtorch.zip -d /

ENV LIBTORCH=/libtorch
ENV LD_LIBRARY_PATH="/libtorch/lib:$LD_LIBRARY_PATH"

COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release

COPY . .

#COPY ./orm .
#COPY ./server .
#COPY ./Cargo.lock .
#COPY ./Cargo.toml .

RUN cargo build --release
RUN mv ./target/release/server ./app

FROM node as frontend_builder

WORKDIR /app

COPY ./frontend/package.json ./frontend/package-lock.json ./

RUN npm ci

COPY ./frontend .

RUN npm run build

FROM fedora:latest as runtime

RUN dnf update -y
RUN dnf install ca-certificates wget unzip bzip2-libs -y

# Fedora's bzip2-libs ships SONAME `libbz2.so.1`, but libtorch_cpu.so asks for
# `libbz2.so.1.0`. Symlink the two so the dynamic linker is happy.
RUN ln -sf /usr/lib64/libbz2.so.1.0.8 /usr/lib64/libbz2.so.1.0 && ldconfig

# Copy libtorch from builder
COPY --from=builder /libtorch /libtorch

# Set libtorch env variables
ENV LIBTORCH=/libtorch
ENV LD_LIBRARY_PATH=/libtorch/lib

WORKDIR /usr/local/bin
COPY --from=builder /app/app .
COPY --from=frontend_builder /app/dist ./static

EXPOSE 8080

ENTRYPOINT ["./app"]

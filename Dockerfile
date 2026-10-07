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

# Install libtorch
RUN curl -L https://download.pytorch.org/libtorch/cu121/libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcu121.zip -o libtorch.zip
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
RUN dnf install ca-certificates wget unzip -y

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

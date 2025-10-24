FROM rust:1.90 as builder
WORKDIR /usr/global-ip-announcer
COPY . .
RUN cargo install --path .

FROM ubuntu:24.04
RUN apt update && apt install -y dnsutils && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/global-ip-announcer /usr/local/bin/global-ip-announcer
CMD ["global-ip-announcer"]
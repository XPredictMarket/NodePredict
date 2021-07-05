FROM ubuntu:20.04 AS builder

WORKDIR /build
ADD . /build/

RUN \
    apt-get update && apt-get install curl make build-essential llvm clang -y && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    export PATH="/root/.cargo/bin:$PATH" && \
    make init && \
    make build

FROM ubuntu:20.04
COPY --from=builder /build/target/release/node-predict /usr/bin/
COPY --from=builder /build/target/release/node-predict-dev /usr/bin/
CMD ["/usr/bin/node-predict-dev", "--dev", "--tmp"]


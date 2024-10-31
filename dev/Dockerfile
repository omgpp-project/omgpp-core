# Build Stage
FROM debian:bullseye AS build
USER 0:0
WORKDIR /home/rust

RUN yes | apt update
RUN yes | apt install python3-pip
RUN yes | apt install build-essential
RUN yes | apt install curl
RUN yes | apt install clang
RUN yes | apt install -y protobuf-compiler
RUN YES | apt install libssl-dev
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rust_install
RUN chmod +x rust_install
RUN ./rust_install -y
RUN . ~/.cargo/env
RUN ln -s ~/.cargo/bin/cargo /bin/cargo
RUN ln -s ~/.cargo/bin/rustup /bin/rustup

RUN rustup component add rust-analyzer

CMD ["/bin/sleep","infinity"]

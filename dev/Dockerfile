# Build Stage
FROM archlinux AS build
USER 0:0
WORKDIR /home/rust

RUN yes | pacman -Sy clang
RUN yes | pacman -S openssl protobuf abseil-cpp curl


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rust_install
RUN chmod +x rust_install
RUN ./rust_install -y
RUN . ~/.cargo/env
RUN ln -s ~/.cargo/bin/cargo /bin/cargo
RUN ln -s ~/.cargo/bin/rustup /bin/rustup

RUN rustup component add rust-analyzer
CMD ["/bin/sleep","infinity"]


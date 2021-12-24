FROM rust:bullseye

USER root
RUN apt update
RUN apt install -y pkg-config libudev-dev mingw-w64

RUN rustup target add x86_64-pc-windows-gnu
RUN cargo install cargo-make

WORKDIR /tmp
RUN mkdir build

WORKDIR /tmp/build

ADD ./run.sh run.sh
RUN chmod +x run.sh

ENTRYPOINT [ "./run.sh" ]

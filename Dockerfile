FROM rust:bullseye

RUN apt update
RUN apt install -y pkg-config libudev-dev mingw-w64

RUN rustup target add x86_64-pc-windows-gnu
RUN cargo install cargo-make

WORKDIR /root
RUN mkdir build

WORKDIR /root/build

ENTRYPOINT [ "cargo", "make", "sudoku" ]

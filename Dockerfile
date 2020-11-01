FROM rust:1 AS build

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml /globe/
COPY ./globe /globe/globe
COPY ./globe-cli /globe/globe-cli
WORKDIR /globe/

RUN cargo install --target x86_64-unknown-linux-musl --path ./globe-cli --root /globe/bin/

FROM scratch

COPY --from=build /globe/bin/bin/globe /
ENTRYPOINT [ "/globe" ]

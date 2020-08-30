FROM ekidd/rust-musl-builder:stable as builder

# build depedencies first, that way they will be cached
# there is a very high chance that project's files will be updated rather than dependencies

# note: this will be useful if you build it locally, on CI it builds from scratch every time.
RUN USER=rust cargo new --bin epitech-ics
WORKDIR ./epitech-ics
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*

# build actual project
ADD src/ ./src/
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/epitech_ics*
RUN cargo build --release

# now we can prepare to run the project
FROM alpine:latest

ARG APP_PATH=/usr/src/app

# copy binary to final folder
COPY --from=builder /home/rust/src/epitech-ics/target/x86_64-unknown-linux-musl/release/epitech-ics ${APP_PATH}/epitech-ics

WORKDIR ${APP_PATH}

EXPOSE 4343
CMD ["./epitech-ics"]
# use rust alpine container for building
FROM docker.io/rust:1.77-alpine AS build

# install alpine packages
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# fetch dependencies
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && touch src/main.rs && cargo fetch

# compile binary
COPY Rocket.toml ./
COPY src src
COPY migrations migrations
RUN cargo build --release

# use fresh alpine as basis for final image
FROM docker.io/alpine:3.19

# add compiled binary
COPY --from=build src/target/release/backend-template /usr/local/bin/

# add static assets
COPY static /usr/local/share/backend

# set execution environment
ENV ROCKET_ADDRESS="0.0.0.0"
USER 1000:1000
EXPOSE 8000
ENTRYPOINT ["backend-template"]

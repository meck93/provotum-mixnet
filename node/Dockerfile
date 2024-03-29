FROM debian:stretch-slim AS builder

WORKDIR /provotum

RUN apt-get -y update && \
  apt-get install -y \
  software-properties-common curl git file binutils binutils-dev \
  make cmake ca-certificates gcc g++ zip dpkg-dev python openssl gettext \
  build-essential pkg-config libssl-dev libudev-dev time clang llvm-dev

# install rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# rustup directory
ENV PATH /root/.cargo/bin:$PATH
ENV CARGO_TERM_COLOR=always

# setup rust nightly channel
RUN rustup update
RUN rustup toolchain install nightly-2021-01-20

# install wasm toolchain for substrate
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-20

#compiler ENV
ENV CC gcc
ENV CXX g++

# Copy code to build directory, instead of only using .dockerignore, we copy elements
# explicitly. This lets us cache build results while iterating on scripts.
# Currently, the docker image can only be built from the root folder because the crypto crate needs to be copied manually.
# TODO: remove as soon as project is public + change the way dependency is referenced (remove path, add Github).
COPY ./crypto ./crypto
COPY ./node/runtime ./node/runtime
COPY ./node/node ./node/node
COPY ./node/pallets ./node/pallets
COPY ./node/Cargo.toml ./node

# Cache the result of the fetch in case the build gets interrupted
WORKDIR /provotum/node
RUN cargo fetch 
# Build node.
RUN cargo build --release --color=always

# Final stage. Copy the node executable and the script
FROM debian:stretch-slim

LABEL org.opencontainers.image.source https://github.com/meck93/provotum-mixnet

WORKDIR /provotum

COPY --from=builder /provotum/node/target/release/provotum .

# curl is required for uploading to keystore
# note: `subkey insert` is a potential alternarve to curl
RUN apt-get -y update \
  && apt-get install -y --no-install-recommends curl ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  # apt cleanup
  && apt-get autoremove -y \
  && apt-get clean \
  && find /var/lib/apt/lists/ -type f -not -name lock -delete;

# add non root user
RUN useradd -m -u 1000 -U -s /bin/sh -d /provotum provotum-user

# set required env variables
ENV PATH /provotum:$PATH
ENV CARGO_TERM_COLOR=always
ENV RUST_BACKTRACE=1

# set the user
USER provotum-user

# check if executable works in this container
RUN provotum --version

# expose node ports
EXPOSE 30333 9933 9944

ENTRYPOINT ["provotum"]
CMD ["--dev", "--ws-external", "--rpc-external"]
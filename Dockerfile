FROM rust

WORKDIR /tmp

COPY . .
ENV RUST_BACKTRACE=1
CMD [ "cargo", "build" ]

WORKDIR /
CMD [ "/tmp/target/release/quick-size" ]

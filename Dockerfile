FROM rust:slim-bullseye AS builder

# Build app
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release && mv ./target/release/bogdanfloris-com ./bogdanfloris-com

# Runtime image
FROM debian:bullseye-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries and css file from builder's cargo install directory
COPY --from=builder /usr/src/app/bogdanfloris-com /app/bogdanfloris-com
COPY --from=builder /usr/src/app/dist /app/dist
COPY --from=builder /usr/src/app/blog_posts /app/blog_posts

# Run the app
CMD ["./bogdanfloris-com"]


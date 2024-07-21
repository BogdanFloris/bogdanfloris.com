FROM rust:slim-bullseye AS builder

# Build app
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release && mv ./target/release/bogdanfloris-com ./bogdanfloris-com

# Build the tailwindcss output file
RUN apt-get update && apt-get install -y --no-install-recommends curl sqlite3 \
    && curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
    chmod +x tailwindcss-linux-x64 && \
    mv tailwindcss-linux-x64 tailwindcss \
    && ./tailwindcss -i src/style.css -o dist/output.css

# Runtime image
FROM debian:bullseye-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries and css file from builder's cargo install directory
COPY --from=builder /usr/src/app/bogdanfloris-com /app/bogdanfloris-com
COPY --from=builder /usr/src/app/dist/output.css /app/dist/output.css
COPY --from=builder /usr/src/app/blog_posts /app/blog_posts

# Run the app
CMD ["./bogdanfloris-com"]


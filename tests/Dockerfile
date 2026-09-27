FROM node:24-bookworm AS node
FROM rust:1.98.1-bookworm

COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/npm
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

ENV WRANGLER_SEND_METRICS=false
WORKDIR /app

COPY rust-toolchain.toml ./
RUN rustup show active-toolchain \
    && cargo install worker-build --version 0.8.7 --locked
RUN npm install --no-save --package-lock=false wrangler@4.142.0

COPY Cargo.toml Cargo.lock wrangler.toml ./
COPY src ./src
RUN npx wrangler deploy --dry-run --keep-vars

COPY tests ./tests
CMD ["sh", "-c", "cargo test --locked && cargo clippy --locked --all-targets -- -D warnings && node --test tests/shortlinks.test.mjs"]

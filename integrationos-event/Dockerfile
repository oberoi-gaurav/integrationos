# syntax = devthefuture/dockerfile-x

ARG EXECUTABLE=integrationos-event
INCLUDE Dockerfile.common

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/integrationos-event/target/release/integrationos-event /usr/local/bin
ENTRYPOINT /usr/local/bin/integrationos-event

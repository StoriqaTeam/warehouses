FROM debian:stable-slim

ARG env=debug

RUN apt-get update \
  && apt-get install -y wget gnupg2 \
  && sh -c 'wget -q https://www.postgresql.org/media/keys/ACCC4CF8.asc -O - | apt-key add -' \
  && sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt/ stretch-pgdg main" >> /etc/apt/sources.list.d/pgdg.list' \
  && apt-get update \
  && apt-get install -y libpq5 libmariadbclient18 \
  && wget -q https://s3.eu-central-1.amazonaws.com/dumpster.stq/diesel -O /usr/local/bin/diesel \
  && chmod +x /usr/local/bin/diesel \
  && apt-get purge -y wget \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/ \
  && adduser --disabled-password --gecos "" --home /app -u 5000 app \
  && mkdir -p /app/config \
  && mkdir -p /app/migrations \
  && chown -R app: /app

COPY target/$env/warehouses /app
COPY Cargo.toml /app/Cargo.toml
COPY config /app/config
COPY migrations /app/migrations

USER app
WORKDIR /app
EXPOSE 8000

ENTRYPOINT diesel migration run && /app/warehouses

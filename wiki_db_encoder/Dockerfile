FROM rust:1

WORKDIR /usr/src/wiki_db_encoder
COPY . .

RUN cargo install --path .

CMD ["wiki_db_encoder"]
FROM rust:1

WORKDIR /usr/src/wiki_game_solver_web
COPY . .

RUN cargo install --path .

CMD ["wiki_game_solver_web"]
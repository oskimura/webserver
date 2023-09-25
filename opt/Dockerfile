FROM --platform=linux/x86_64 rust

# WORKDIR /usr/src/myapp

WORKDIR /usr/src/myapp
COPY . .
#COPY ./practice-rust ./practice-rust
#RUN ls
#COPY ./practice-rust .
WORKDIR /usr/src/myapp/practice-rust
#COPY . .
#RUN ls
#RUN rustup component add rustfmt
RUN cargo install --target=x86_64-unknown-linux-gnu --path .


EXPOSE 8080
#COPY /usr/local/cargo/bin/practice-rust /usr/local/bin/practice-rust
#COPY --from=builder /usr/local/cargo/bin/practice-rust /usr/local/bin/practice-rust

CMD [ "/usr/local/cargo/bin/practice-rust" ]
#CMD [ "cargo", "run", "-r"]

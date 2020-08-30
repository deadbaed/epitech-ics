# epitech-ics

## development

Tools required: `rust` and `cargo`. You can use [rustup](https://rustup.rs) to install them.

Run `cargo build` to compile and `cargo run` to start the web server.

By default, the server listens on port `4343`. Change this value with the `PORT` environment variable.

⚠️ Warning: The server listens on http only, which means that zero bytes will be encrypted!
There is confidential data that will be transferred between the client and the server, please keep security in mind when deploying.

## deployment

Run `cargo build --release` to compile with optimisations enabled.

The binary will be available at `./target/release/epitech-ics`.

You can also deploy with Docker by building the [Dockerfile](Dockerfile) and using it.

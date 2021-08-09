FROM gitpod/workspace-full

ENV CARGO_HOME=/home/gitpod/.cargo

RUN bash -cl "rustup default stable-2020-10-08-x86_64-unknown-linux-gnu && rustup target add wasm32-unknown-unknown"

RUN bash -c ". .nvm/nvm.sh \
             && nvm install v12 && nvm alias default v12"
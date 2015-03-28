FROM timhabermaas/rust-buildbox
MAINTAINER Tim Habermaas

RUN apt-get -qq update && apt-get -qqy install unzip
RUN mkdir -p /home/rust/wca-api
ADD ./src /home/rust/wca-api/src
ADD Cargo.toml /home/rust/wca-api/
ADD Cargo.lock /home/rust/wca-api/
WORKDIR /home/rust/wca-api
RUN cargo build --release
EXPOSE 3000
VOLUME ["/home/rust/wca-api/data"]

[![Build Status](https://travis-ci.org/cubing/wca-documents-extra.svg?branch=master)](https://travis-ci.org/cubing/wca-documents-extra)

# WCA API written in Rust

## Run using Docker

```
docker run -v path-to-data-directory:/home/rust/wca-api/data -p 80:3000 -d  timhabermaas/wca-api-rust
```

This will download the docker image from Docker Hub and make the app accessible through port 80 on localhost. You can also build the image yourself using

```
docker build -t some-user/some-repo .
```

## Run without Docker

You need [Rust](http://www.rust-lang.org/install.htm) in order to compile and run it using

```
cargo run --release
```

Then access the API through [http://localhost:3000](http://localhost:3000).

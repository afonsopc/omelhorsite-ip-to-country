# IP to Country API

This is a simple API that maps IP addresses to countries. It's written in Rust and uses the Actix web framework.

## Project Structure

- `ip-to-country/`: This directory contains the Rust source code for the API.
  - `src/main.rs`: This is the main entry point for the API.
- `mmdb/`: This directory contains the MaxMind GeoLite2 database file (`database.mmdb`) that the API uses to map IP addresses to countries.
- `Dockerfile`: This file is used to build a Docker image for the API.
- `docker-compose.yml`: This file is used to define and run the API's Docker container.

## How to Run

1. Build the Docker image:

```sh
docker build -t ip-to-country .
```

2. Run the Docker container:

```sh
docker compose up
```

The API will be available at `http://localhost:3000`.

## API Usage

To get the country for an IP address, send a GET request to `http://localhost:3000/{ip}`. For example:

```sh
curl http://localhost:3000/209.51.188.116
```

This will return a JSON object with the country code and country name for the IP address.

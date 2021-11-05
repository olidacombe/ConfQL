# Example

This is an example webserver which parses `schema.gql` to expose a graphql endpoint which resolves data from a directory on the filesystem.

Please see the included `schema.gql` and `data` directory to get going.

## Making a Docker image

Assemble your schema file, a data directory, and [this Dockerfile](Dockerfile) like so:

```
.
├── Dockerfile
├── data
│   └── id.yml # and probably a lot more ;)
└── schema.gql
```

Then build and run:

```bash
docker build -t my-server .
docker run -v $(pwd)/data:/data -p 8080:8080 my-server
```

## Running Locally

```
cargo run
```

## Environment Variables

The following variables configure the server:

| Variable | |
|-|-|
| BIND_ADDR | Bind address, default `0.0.0.0` |
| DATA_ROOT | Root path of directory containing yaml data to serve, default is current working directory |
| PORT | TCP Port to listen on, default `8080` |
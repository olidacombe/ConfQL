# Example

This is an example webserver which parses `schema.gql` to expose a graphql endpoint, resolving data from a directory on the filesystem.

Please see the included `schema.gql` and `data` directory to get going.

For more detailed on special schema directives or otherwise how things work, please consult the [rust docs](https://docs.rs/confql) or [repo root](..).

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

Then query the endpoint at `127.0.0.1:8080/graphql` using your favourite GraphQL
client or `curl`, e.g.

```bash
curl -g \
	-X POST \
	-H "Content-Type: application/json" \
	-d '{"query":"query{id}"}' \
	http://127.0.0.1:8080/graphql
```

## Running Locally

```
DATA_ROOT=data cargo run
```

Then query the endpoint at `127.0.0.1:8080/graphql` using your favourite GraphQL
client or `curl`, e.g.

```bash
curl -g \
	-X POST \
	-H "Content-Type: application/json" \
	-d '{"query":"query{id}"}' \
	http://127.0.0.1:8080/graphql
```

## Environment Variables

The following variables configure the server:

| Variable | |
|-|-|
| BIND_ADDR | Bind address, default `0.0.0.0` |
| DATA_ROOT | Root path of directory containing yaml data to serve, default is current working directory |
| PORT | TCP Port to listen on, default `8080` |

## Schema Changes

Data is read on the fly, but if you change your schema, that needs a recompile.  [ConfQL](..) is essentially a procedural macro which bakes data file traversal impls at compile time from your schema definition.
# ConfQL

This is intended as a very low-friction means of turning structured yaml into a [GraphQL](https://graphql.org/) service. If you have a directory containing structured yaml, I want it to be as simple as this:

- create a [schema](https://graphql.org/learn/schema/) file formalizing your data structure
- drop in a [Dockerfile](example/Dockerfile)
- build a container and run

See the [example](example) for a quick start.

### Motivation

I deal with _infrastructure as code_ a lot, and have found myself writing boilerplate to consume a repo full of yaml for various purposes. It makes more sense to me to expose the whole repo as a GraphQL service, and pull out what I need using a query (say with a terraform [data provider](https://registry.terraform.io/providers/sullivtr/graphql/latest/docs/data-sources/query)). This seems like something that could be useful in other contexts.

## What Does it Do?

Suppose you have this schema:

```gql
type A {
  b: B!
}

type B {
  c: C!
}

type C {
  id: Int!
  name: String!
  tags: [String!]
}

type Query {
  a: A!
}

schema {
  query: Query
}
```

And these files:

```yml
---
# index.yml
a:
  b:
    c:
      id: 14
```

```yml
---
# a/b.yml
c:
  name: Biggy
  tags:
    - Boss
```

```yml
---
# a/b/c/index.yml
tags:
  - Big Shot
  - Winner
```

```yml
---
# a/b/c/tags.yml
- Inspiration
- Mentor
```

...or any other ridiculous combination you can think of.

Then all the data should get appropriately merged so you can query:

```gql
{
  a {
    b {
      c {
        id
        name
        tags
      }
    }
  }
}
```

to get

```json
{
  "data": {
    "a": {
      "b": {
        "c": {
          "id": 14,
          "name": "Biggy",
          "tags": ["Boss", "Big Shot", "Winner", "Inspiration", "Mentor"]
        }
      }
    }
  }
}
```

## Special Directives

### `arrayIdentifier`

The yaml file use case threw up a common pattern where there's an _array_ of objects represented by a _directory_ of yaml files, or a _mapping_ of objects, where each _filename_ or _key_ respectively logically represents a unique identifier field within each object.

E.g.

```
teams
├── backend.yml
├── frontend.yml
├── qa.yml
└── sre.yml
```

If you want the filename mapped to a field in your GraphQL type, you can use the `@confql(arrayIdentifier: true)` directive. E.g.

```gql
type User {
  name: String!
  email: String!
}

type Team {
  name: String! @confql(arrayIdentifier: true)
  members: [User!]!
}

type Query {
  teams: [Team!]!
}

schema {
  query: Query
}
```

Then, in the above case, the strings `backend`, `frontend`, `qa`, `sre` would get mapped to the `name` field of each `Team` in `teams`.

This also works with directory names if you've broken your data up further, e.g.

```
teams
├── backend
│   ├── index.yml
│   └── members.yml
├── frontend.yml
├── qa.yml
└── sre.yml
```

Similarly, you can get the same effect from a mapping:

```yml
---
# teams.yml
backend:
  members:
    - name: Bill
      email: bill@ho.me
frontend:
  members:
    - name: Will
      email: will@ho.me
# etc.
```

### otherThing

```gql
type User {
  name: String!
  email: String!
}

type Team {
  name: String! @confql(arrayIdentifier: true)
  members: [User!]!
}

type Query {
  teams: [Team!]!
  users: [User!]!
  user($email: String!): User @confql(findIn: "users", by: "email")
  teamsOf($email: String!): [Team!]! @confql(filter: "teams", by: "members.email")
}

schema {
  query: Query
}
```

## How Does it Work?

At its heart, this is a [procedural macro](https://doc.rust-lang.org/reference/procedural-macros.html) which takes a path to a schema file, and at compile-time generates a [juniper](https://graphql-rust.github.io/juniper/master/index.html) server with all necessary functionality to resolve data from the filesystem adhering to the given schema. It is draws much inspiration from, and is much more basic than [juniper-from-schema](https://github.com/davidpdrsn/juniper-from-schema).


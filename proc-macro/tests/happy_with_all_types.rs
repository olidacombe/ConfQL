use confql::graphql_schema;

graphql_schema! {
    type Query {
        optional_string: String
        string: String!
        optional_vec_optional_string: [String]
        optional_vec_string: [String!]
        vec_optional_string: [String]!
        vec_string: [String!]!
        optional_float: Float
        float: Float!
        optional_vec_optional_float: [Float]
        optional_vec_float: [Float!]
        vec_optional_float: [Float]!
        vec_float: [Float!]!
        optional_int: Int
        int: Int!
        optional_vec_optional_int: [Int]
        optional_vec_int: [Int!]
        vec_optional_int: [Int]!
        vec_int: [Int!]!
        optional_boolean: Boolean
        boolean: Boolean!
        optional_vec_optional_boolean: [Boolean]
        optional_vec_boolean: [Boolean!]
        vec_optional_boolean: [Boolean]!
        vec_boolean: [Boolean!]!
        optional_id: ID
        id: ID!
        optional_vec_optional_id: [ID]
        optional_vec_id: [ID!]
        vec_optional_id: [ID]!
        vec_id: [ID!]!
    }

    schema {
        query: Query
    }
}

fn main() {}

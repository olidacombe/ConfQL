//! Actix web example
//!
//! A simple example integrating with actix-web
#[macro_use]
extern crate lazy_static;
use std::io;
use std::sync::Arc;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use confql::graphql_schema;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::{EmptyMutation, EmptySubscription};

graphql_schema! {
    type Query {
        id: String!
    }

    schema {
        query: Query
    }
}

lazy_static! {
    static ref BIND_ADDR: String = std::env::var("BIND_ADDR")
        .ok()
        .unwrap_or("127.0.0.1".to_string());
    static ref PORT: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    static ref ADDR: String = format!("{}:{}", *BIND_ADDR, *PORT);
    static ref DATA_ROOT: PathBuf = std::env::var("DATA_ROOT")
        .map_or_else(|_e| std::env::current_dir().unwrap(), |root| root.into())
        .canonicalize()
        .unwrap();
    static ref CTX: Ctx = Ctx::from(DATA_ROOT.clone());
}

async fn graphiql() -> HttpResponse {
    let html = graphiql_source(&format!("http://{}/graphql", *ADDR), None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute_sync(&st, &CTX);
        serde_json::to_string(&res)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create Juniper schema
    let schema = std::sync::Arc::new(Schema::new(
        Query,
        EmptyMutation::new(),
        EmptySubscription::new(),
    ));

    log::info!(
        "Starting webserver {} from data path {:?}",
        *ADDR,
        *DATA_ROOT
    );

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind(&*ADDR)?
    .run()
    .await
}

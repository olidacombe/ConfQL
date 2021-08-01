use async_graphql::SimpleObject;
use serde::Deserialize;

#[derive(SimpleObject, Deserialize, Debug)]
pub struct Hero {
    pub name: String,
    pub id: u32,
    pub powers: Vec<String>,
}

mod intercept;
mod test_utils; // Yes, i can just make it cfg!(test), but then the dependencies of intercept.rs
                // will be not as compact
mod text;
use std::error::Error;

use dotenvy::dotenv;
use teloxide::dispatching::dialogue::serializer::Cbor;
use teloxide::dispatching::dialogue::{Dialogue, ErasedStorage, RedisStorage, Storage};
use teloxide::prelude::*;

mod bot_schema;
mod handlers;

pub type MyDialogue = Dialogue<State, ErasedStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;
pub type MyStorage = std::sync::Arc<ErasedStorage<State>>;

#[derive(Clone, PartialEq, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start, // The default state, from which you can send '/start'
    WhatDoYouWant, // We ask, what do you want, to add or subtract
    GetFirstNumber {
        // We got what the user wants to do, and we ask for the first number
        operation: String,
    },
    GetSecondNumber {
        // Now ask for the second number
        first_number: i32,
        operation: String,
    },
}

pub async fn get_bot_storage() -> MyStorage {
    let storage: MyStorage = RedisStorage::open(dotenvy::var("REDIS_URL").unwrap(), Cbor)
        // For reasons unknown to me, Binary serializer doesn't accept json-like objects,
        // Message in particular, so im using it. Also, i don't think that InMemStorage
        // can work with tests, we need the storage to be persistent
        .await
        .unwrap()
        .erase();
    storage
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();

    Dispatcher::builder(bot, bot_schema::schema())
        .dependencies(dptree::deps![get_bot_storage().await])
        .build()
        .dispatch()
        .await;
}

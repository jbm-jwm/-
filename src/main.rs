#[macro_use]
extern crate reqwest;
extern crate scraper;

use std::env;
mod messaging;

use serenity::prelude::*;

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::new(
        &token,
        messaging::message::Handler::new(),
    )
    .expect("Err creating client");
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

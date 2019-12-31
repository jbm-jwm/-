use crate::messaging::jisho::jisho_handler;
use phf::phf_map;
use serenity::{
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};

const BOTNAME: &str = "トゥアール";

type Callback = fn(&str, &reqwest::Client, ctx: Context, msg: &ChannelId);
static CMDDISPATCHER: phf::Map<&'static str, Callback> = phf_map! {
    "!music" => music_handler,
    "!cpp" => cpp_handler,
    "!jisho" => jisho_handler,
};
fn find_cmp(name: &str) -> Option<Callback> {
    CMDDISPATCHER.get(name.to_lowercase().as_str()).cloned()
}

fn music_handler(args: &str, _client: &reqwest::Client, ctx: Context, msg: &ChannelId) {
    // TODO Remove hard coded links
    let ret = match args {
        "ツインテール" => "https://www.youtube.com/watch?v=6lyg37Tp8FE",
        "下ネタ" => "https://www.youtube.com/watch?v=hfBY2E3snFs&feature=youtu.be",
        "下ネタba" => "https://www.youtube.com/watch?v=PAZmHjICc3U&feature=youtu.be",
        _ => "valid songs are ツインテール, 下ネタ, 下ネタba",
    };
    match msg.say(ctx.http, ret) {
        Ok(_) => {}
        Err(e) => {
            println!("Issue recording delete: {:?}", e);
        }
    }
}

fn cpp_handler(_args: &str, _client: &reqwest::Client, ctx: Context, msg: &ChannelId) {
    match msg.say(ctx.http, "Not Implemented") {
        Ok(_) => {}
        Err(e) => {
            println!("Issue recording delete: {:?}", e);
        }
    }
}

pub struct Handler {
    m_client: reqwest::Client,
}
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions.iter().any(|u| u.name == BOTNAME) {
            let token: Vec<&str> = msg.content.split(' ').collect();
            if token.len() >= 3 {
                match find_cmp(token[1]) {
                    Some(value) => value(&token[2..].join(" "), &self.m_client, ctx, &msg.channel_id),
                    None => {
                        let send =
                            format!("Unknown cmd {0} valid cmd are [music,jisho,cpp]", token[1]);
                        match msg.channel_id.say(ctx.http, send) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Issue recording delete: {:?}", e);
                            }
                        }
                    }
                }
            } else {
                let send = format!(
                    "valid format cmd are @{0} !cmd args,\n \
                     where valid cmd are:[music, jisho, cpp]",
                    BOTNAME
                );
                match msg.channel_id.say(ctx.http, send)
		{
                    Ok(_) => {}
                    Err(e) => {
                        println!("Issue recording delete: {:?}", e);
                    }
                }
            }
        }
    }
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
impl Handler {
    pub fn new() -> Handler {
        Handler {
            m_client: reqwest::Client::new(),
        }
    }
}

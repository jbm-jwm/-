use crate::messaging::jisho;
use phf::phf_map;
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const BOTNAME: &str = "トゥアール";

type Callback = fn(&str, &reqwest::Client) -> String;

static CMDDISPATCHER: phf::Map<&'static str, Callback> = phf_map! {
    "!music" => music_handler,
    "!cpp" => cpp_handler,
    "!jisho" => jisho_handler,
};

fn find_cmp(name: &str) -> Option<Callback> {
    CMDDISPATCHER.get(name.to_lowercase().as_str()).cloned()
}

fn music_handler(args: &str, _client : &reqwest::Client) -> String {
    // TODO Remove hard coded links
    let ret = match args {
        "ツインテール" => "https://www.youtube.com/watch?v=6lyg37Tp8FE",
        "下ネタ" => "https://www.youtube.com/watch?v=hfBY2E3snFs&feature=youtu.be",
        "下ネタba" => "https://www.youtube.com/watch?v=PAZmHjICc3U&feature=youtu.be",
        _ => "valid songs are ツインテール, 下ネタ, 下ネタba",
    };
    String::from(ret)
}

fn jisho_handler(args: &str, client : &reqwest::Client) -> String {
    let token: Vec<&str> = args.split_whitespace().collect();
    let sendme: String = match token.len() {
        2 => {
            if let Ok(value) = jisho::MessageInfo::generate_msg(token[1], client) {
                match token[0] {
                    "english" => value.m_senses.join("-"),
                    "reading" => value.m_reading.join("-"),
                    "jlpt" => match value.m_jlpt {
                        Some(value) => value,
                        None => String::from("None"),
                    },
                    "all" => format!(
                        "{0}\n{1}\n{2}\n{3}\n",
                        value.m_japanese,
                        value.m_reading.join("-"),
                        value.m_senses.join("-"),
                        match value.m_jlpt {
                            Some(value) => value,
                            None => String::from("None"),
                        }
                    ),
                    _ => String::from("unsupported command"),
                }
            } else {
                String::from("Kanji not found please report if it is a bug")
            }
        }
        _ => String::from(
            "valid command are !jisho cmd kanji, with cmd in [all,reading,jlpt,english]",
        ),
    };
    format!("```{}```",sendme)
}
fn cpp_handler(_args: &str, _client: &reqwest::Client) -> String {
    "Not Implemented".to_string()
}

pub struct Handler{
    m_client : reqwest::Client,
}
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.mentions.iter().any(|u| u.name == BOTNAME) {
            let token: Vec<&str> = msg.content.splitn(3, ' ').collect();
            let args = match token.len() {
                1 => Err("missing cmd parameter"),
                2 | 3 => Ok(token[2]),
                _ => Ok("help"),
            };
            let send = match args {
                Ok(args) => match find_cmp(token[1]) {
                    Some(value) => value(args, &self.m_client),
                    None => format!("Unknown cmd {0} valid cmd are [music,jisho,cpp]", token[1]),
                },
                Err(_args) => format!(
                    "valid format cmd are @{0} !cmd args,\n \
                     where valid cmd are:[music, jisho, cpp]",
                    BOTNAME
                ),
            };
            if let Err(why) = msg.channel_id.say(&ctx.http, send) {
                println!("Error sending message: {:?}", why);
            }
        }
    }
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
impl Handler {
    pub fn new() -> Handler {
        Handler {m_client : reqwest::Client::new()}
    }
}

use serenity::{
    model::id::ChannelId,
    prelude::*,
};

#[derive(Deserialize,Debug)]
struct Senses {
    english_definitions: Vec<String>,
}

#[derive(Deserialize,Debug)]
struct Lecture {
    word: Option<String>,
    reading: Option<String>,
}
#[derive(Deserialize,Debug)]
struct Definition {
    japanese: Vec<Lecture>,
    senses: Vec<Senses>,
    jlpt: Vec<String>,
}

#[derive(Deserialize,Debug)]
struct Jisho {
    data: Vec<Definition>,
}
#[derive(Debug)]
pub struct MessageInfo {
    pub m_japanese: String,
    pub m_reading: Vec<String>,
    pub m_jlpt: Option<String>,
    pub m_senses: Vec<String>,
}
impl MessageInfo {
    pub fn new(kanji: &str) -> MessageInfo {
        MessageInfo {
            m_japanese: kanji.to_string(),
            m_jlpt: None,
            m_reading: Vec::new(),
            m_senses: Vec::new(),
        }
    }
    pub fn generate_msg(kanji: &str, client: &reqwest::Client) -> Result<MessageInfo, &'static str> {
        let url = format!(
            "{}{}",
            "https://jisho.org/api/v1/search/words?keyword=", kanji
        );
        let mut response = client
            .get(reqwest::Url::parse(&url).unwrap())
            .send()
            .expect("Failed to send request");
        let mut msg = MessageInfo::new(kanji);
        if let Ok(jisho_word) = response.json::<Jisho>() {
            for data in jisho_word.data.into_iter() {
                if let Some(word) = data.japanese.into_iter().find(|elt| match &elt.word {
                    Some(value) => value == kanji,
                    None => false,
                }) {
                    for elts in data.senses.into_iter() {
                        msg.m_senses.extend(elts.english_definitions);
                    }
                    match word.reading {
                        Some(value) => msg.m_reading.push(value),
                        None => () ,
                    }
                    msg.m_jlpt = match data.jlpt.len() {
                        0 => msg.m_jlpt,
                        _ => Some(data.jlpt[0].clone()),
                    };
                }
            }
        } else {
            println!("Error");
        }
        match msg.m_reading.is_empty() {
            true => Err("not found"),
            false => Ok(msg),
        }
    }
}


pub fn jisho_handler(args: &str, client: &reqwest::Client, ctx: Context, msg: &ChannelId) {
    if let Ok(value) = MessageInfo::generate_msg(args, client) {
        if let Ok(_val) = msg.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(args);
                e.description(value.m_senses.join(","));
                e.field("Reading", value.m_reading.join(","), false);
                e.field(
                    "Jlpt",
                    match value.m_jlpt {
                        Some(value) => value,
                        None => String::from("None"),
                    },
                    false,
                );
                e.color(serenity::utils::Colour::from_rgb(81,175,239));
                e.url("https://jisho.org/search/%E8%8A%B1%23kanji");
                e
            });
            m
        }) {
        } else {
            match msg.say(ctx.http, "Kanji not found please report if it is a bug") {
                Ok(_) => {}
                Err(e) => {
                    println!("Issue recording delete: {:?}", e);
                }
            }
        }
    };
}

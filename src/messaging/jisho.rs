use scraper::{Html, Selector};
use serenity::{model::id::ChannelId, prelude::*};
use std::collections::HashMap;
static CSS_KANJI: [&str; 6] = [
    ".character",
    ".kanji-details__main-meanings",
    ".jlpt",
    ".dictionary_entry.kun_yomi",
    ".dictionary_entry.variants",
    ".dictionary_entry.on_yomi", // /!\ on yomi contains radicals parts so done on the end
];

static JISHO_KEY: [&str; 6] = ["Kanji", "English", "Jlpt", "Kun", "Variants", "On"];
static MESSAGE_KEY: [&str; 6] = ["Variants", "Parts", "On", "Kun", "Jlpt", "Radical"];

pub fn generate_msg(
    kanji: &str,
    client: &reqwest::Client,
) -> std::result::Result<HashMap<&'static str, String>, &'static str> {
    let url = format!("{}{}%23kanji", "https://jisho.org/search/", kanji);
    let mut field_lookup: HashMap<&'static str, String> = HashMap::new();
    let mut resp = client
        .get(reqwest::Url::parse(&url).unwrap())
        .send()
        .expect("Failed to send request");
    if let Ok(body) = resp.text() {
        // parses string of HTML as a document
        let fragment = Html::parse_document(&body);
        // parses based on a CSS selector
        for (css_it, key_it) in CSS_KANJI.iter().zip(JISHO_KEY.iter()) {
            let extract = Selector::parse(css_it).unwrap();
            // iterate over elements matching our selector
            for field in fragment.select(&extract) {
                // grab the headline text and place into a vector
                let mut field_txt = field.text().collect::<Vec<&str>>();
                field_txt.retain(|&elt| !elt.trim().is_empty());
                // format to struct
                let key = match field_txt[0].trim() {
                    "Radical:" => {
                        String::from(field_txt.remove(0).trim());
                        "Radical"
                    }
                    "Parts:" => {
                        String::from(field_txt.remove(0).trim());
                        "Parts"
                    }
                    "Kun:" | "JLPT level" | "Variants:" | "On:" => {
                        String::from(field_txt.remove(0).trim());
                        key_it
                    }
                    // case English|Kanji
                    _ => key_it,
                };
                field_lookup.insert(key, field_txt.join(" ").replace('\n', " "));
            }
        }
        Ok(field_lookup)
    } else {
        Err("Url not found ?")
    }
}

pub struct Definition {
    kanji: String,
    meaning: Vec<String>,
}

pub fn generate_msg_def(kanji: &str, client: &reqwest::Client) -> Vec<Definition> {
    let url = format!("{}{}", "https://jisho.org/search/", kanji);
    //let mut field_lookup: HashMap<&'static str, String> = HashMap::new();
    let mut resp = client
        .get(reqwest::Url::parse(&url).unwrap())
        .send()
        .expect("Failed to send request");
    let body = resp.text().expect("Ooops");
    let fragment = Html::parse_document(&body);
    // first get all kanji
    let mut data: Vec<Definition> = Vec::new();
    for field in  fragment.select(&Selector::parse("div.concept_light.clearfix").unwrap()){
        for field_kanji in field.select(&Selector::parse("span.text").unwrap()) {
            let mut field_txt = field_kanji.text().collect::<Vec<&str>>();
            field_txt.retain(|&elt| !elt.trim().is_empty());
            let kanji = field_txt.join(" ");
            let mut meaning: Vec<String> = Vec::new();
            for field_meaning in field.select(&Selector::parse("div.meaning-definition").unwrap()) {
                let mut field_txt = field_meaning.text().collect::<Vec<&str>>();
                field_txt.retain(|&elt| !elt.trim().is_empty());
                meaning.push(field_txt.join(" "));
            }
            data.push(Definition {
                kanji: kanji,
                meaning: meaning,
            });
        }
    }
    data
}

pub fn jisho_handler(args: &str, client: &reqwest::Client, ctx: Context, msg: &ChannelId) {
    let token: Vec<&str> = args.split(' ').collect();
    match token[0] {
        "kanji" => {
            if let Ok(value) = generate_msg(token[1], client) {
                if let Ok(_mesg) = msg.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(value.get("Kanji").unwrap());
                        e.description(value.get("English").unwrap());
                        for key_it in MESSAGE_KEY.iter() {
                            e.field(
                                key_it,
                                value.get(key_it).unwrap_or(&String::from("None")),
                                false,
                            );
                        }
                        e.color(serenity::utils::Colour::from_rgb(81, 175, 239));
                        e.url(format!(
                            "https://jisho.org/search/{}%23kanji",
                            value.get("Kanji").expect("Ooops"),
                        ));
                        e
                    });
                    m
                }) {}
            } else {
                match msg.say(ctx.http, "Kanji not found please report if it is a bug") {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Issue recording delete: {:?}", e);
                    }
                }
            }
        }
        "word" => {
            let definitions = generate_msg_def(token[1], client);
            for it in definitions.iter() {
                if let Ok(_mesg) = msg.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(it.kanji.clone());
                        e.description(it.meaning[0].clone());
                        e.field("Senses",it.meaning[1..].join("\n"), false);
                        e.color(serenity::utils::Colour::from_rgb(81, 175, 239));
                        e
                    });
                    m
                }) {}
            }
        }
        _ => {}
    }
}

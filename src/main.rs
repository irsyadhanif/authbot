#[macro_use]
extern crate serenity;
extern crate hyper;
extern crate hyper_native_tls;
extern crate regex;
extern crate crypto;

use serenity::client::Context;
use serenity::model::Message;
use serenity::ext::framework::help_commands;
use serenity::utils::MessageBuilder;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use hyper::Client;
use hyper::header::Connection;
use std::io::Read;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use regex::Regex;
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;

fn main() {
    let path = Path::new("key.txt");


    let mut file = match File::open(&path) {
        Err(e) => panic!("Couldn't open key file: {}", e),
        Ok(file) => file,
    };

    let mut key = String::new();
    match file.read_to_string(&mut key){
        Err(e) => panic!("Couldn't read key file: {}", e),
        Ok(key) => key,
    };
    key.pop(); // remove stray newline
    println!("Key is {}", key);

    let mut client = serenity::Client::login_bot(& key);

    client.on_ready(|_ctx, ready| {
        println!("{} is connected!", ready.user.name);
    });

    client.with_framework(|f| f
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .rate_limit_message("Try this again in `%time%` seconds.")
            .prefix("!"))

        .before(|ctx, msg, command_name| {
                println!("Got command '{}' by user '{}'",
                         command_name,
                         msg.author.name);

                true
        })

        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })

        .command("about", |c| c.exec_str("An authentication bot"))
        .command("help", |c| c.exec_help(help_commands::plain))
        .command("getuser", |c| c
            .desc("Takes an FP User ID and returns their details.")
            .exec(getuser))
        .command("beginauth", |c| c
            .desc("Begins the auth process")
            .exec(beginauth))
        .command("userauth", |c| c
            .desc("Continue the auth process")
            .exec(userauth))

    );


    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}











command!(getuser(_ctx, msg, args, first: String) {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = hyper::Client::with_connector(connector);
    if let Err(why) = msg.channel_id.say(&format!("Getting info from user ID {}", first)) {
        println!("Error sending message {:?}", why);
    }

    let url = format!("https://facepunch.com/member.php?u={}#aboutme", first);
    let mut response = client.get(&url).
        header(Connection::close()).send().unwrap();
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    let re = Regex::new(r"(?ixm)<d(d|t)>(.*?)</d(d|t)>").unwrap();
    let reUsername = Regex::new(r"<title>View Profile: (.*?) - Facepunch<\/title>").unwrap();
    let mut results = Vec::new();
    for cap in re.captures_iter(&body) {
        let temp = cap[2].to_string();
        results.push(temp);
    }

    let mut join_date = String::new();
    let mut post_count = String::new();
    let mut location = String::new();
    for (x, result) in results.iter().enumerate() {
        if **result == String::from("Join Date") {
            join_date = results[x + 1].clone();
        } else if **result == String::from("Total Posts") {
            post_count = results[x + 1].clone();
        } else if **result == String::from("Location:") {
            location = results[x + 1].clone();
        }
    }

    if let Err(why) = msg.channel_id.say(&format!("Join date: {}\nPost Count: {}\nToken: {}",
        join_date, post_count, location)) {
        println!("Error sending message {:?}", why);
    }
});

command!(beginauth(_ctx, msg, args) {
    let uid = format!("{:?}",msg.author.id);
    let mut hasher = Sha3::sha3_256();
    if let Err(why) = msg.channel_id.say("To auth, please place the DM'ed key into your location on FP and say !userauth <userid>") {
        println!("Error sending message {:?}", why);
    }
    hasher.input_str(&uid);
    let hash = hasher.result_str();
    if let Err(why) = msg.author.dm(&hash) {
        println!("Error sending message {:?}", why);
    }
});

command!(userauth(_ctx, msg, args, first:String) {
    let uid = format!("{:?}",msg.author.id);
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(&uid);
    let hash = hasher.result_str();

    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = hyper::Client::with_connector(connector);
    let url = format!("https://facepunch.com/member.php?u={}#aboutme", first);
    let mut response = client.get(&url).
        header(Connection::close()).send().unwrap();
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    let re = Regex::new(r"(?ixm)<d(d|t)>(.*?)</d(d|t)>").unwrap();
    let mut results = Vec::new();
    for cap in re.captures_iter(&body) {
        let temp = cap[2].to_string();
        results.push(temp);
    }

    let mut location = String::new();
    for (x, result) in results.iter().enumerate() {
        if **result == String::from("Location:") {
            location = results[x + 1].clone();
        }
    }

    if hash == location {
        if let Err(why) = msg.channel_id.say("User authorized.") {
            println!("Error sending message {:?}", why);
        }
    } else {
        if let Err(why) = msg.channel_id.say("Missing or invalid auth key.") {
            println!("Error sending message {:?}", why);
        }
    }
});




//

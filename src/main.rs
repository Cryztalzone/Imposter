use std::{
    collections::{HashSet},
    io,
    io::BufReader,
    fs::File,
    time::SystemTime,
};
use chrono::{DateTime, Duration, Utc};
use serenity::{
    prelude::*,
    async_trait,
    client::bridge::gateway::GatewayIntents,
    model::{
        channel::{Channel, Message, Embed, EmbedField},
        gateway::{
            Ready,
            Activity,
        },
        id::UserId,
        permissions::Permissions,
    },
    framework::standard::{
        help_commands,
        Args,
        DispatchError,
        HelpOptions,
        CommandGroup,
        CommandResult,
        Reason,
        StandardFramework,
        macros::{
            group,
            help,
            hook,
            command,
        },
    },
    http::Http,
    utils::Colour,
};

use serde::Deserialize;

fn get_config() -> io::Result<Config> {
    let file = File::open("config.json")?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}

#[derive(Deserialize, Debug)]
struct Config {
    token: String,
    application_id: u64,
    version: String,
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::watching("-help @ Version Alpha 2.0")).await;
        //ctx.dnd().await;
        println!("Successfully connected {}", ready.user.name);
    }
}

#[group]
struct General;

#[group]
//#[prefixes("text")]
#[description("A group with commands that make the bot repeat text")]
//#[default_command(echo)]
#[commands(echo, say, whisper)]
struct Text;

#[group]
//#[prefixes("test")]
#[description("A group with commands to test the bot")]
//#[default_command(ping)]
#[commands(active, ping)]
struct Test;

#[group]
//#[prefixes("util")]
#[description("A group with utility commands")]
//#[default_command(changelog)]
#[commands(avatar, default_avatar, changelog, code, count)]
struct Util;

#[group]
#[owners_only]
//#[prefixes("debug")]
#[description("Debug commands")]
#[commands(debug)]
struct Debug;

#[help]
#[command_not_found_text = "Could not find '{}'"]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[wrong_channel = "Strike"]
async fn help(ctx: &Context, msg: &Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("User '{}' issued command '{}'", msg.author.name, command_name);

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Successfully processed command '{}'", command_name),
        Err(reason) => println!("Error processing command '{}'. Reason: {:?}", command_name, reason)
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg.channel_id.say(&ctx.http, &format!("Try again in {} seconds", info.as_secs())).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let config = get_config().expect("Error reading bot token from config");

    let http = Http::new_with_token(&config.token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(reason) => panic!("Error getting bot id: {:?}", reason)
            }
        },
        Err(reason) => panic!("Error getting application info: {:?}", reason)
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(false)
            .on_mention(Some(bot_id))
            .prefix("-") // set the bot's prefix to "-"
            .delimiters(vec![", ", ","])
            .owners(owners))
        .before(before)
        .after(after)
        .on_dispatch_error(dispatch_error)
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&TEXT_GROUP)
        .group(&TEST_GROUP)
        .group(&UTIL_GROUP)
        .group(&DEBUG_GROUP);

    // Login with a bot token
    let mut client = Client::builder(&config.token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(reason) = client.start().await {
        println!("An error occurred while running the client: {:?}", reason);
    }
}

#[command]
#[description = "Repeats your message"]
async fn echo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Cannot send an empty message").await?;
    } else {
        msg.channel_id.say(&ctx.http, &args.rest()).await?;
    }
    Ok(())
}

#[command]
#[description = "Repeats your message with TTS"]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Cannot send an empty message").await?;
    } else {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.content(&args.rest());
            m.tts(true);
        m
        }).await?;
    }
    Ok(())
}

#[command]
#[description = "Deletes your message, then repeats it with TTS (leaves no trace of the author)"]
async fn whisper(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.delete(&ctx.http).await?;
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, "Cannot send an empty message").await?;
    } else {
        say(&ctx, &msg, args).await?;
    }
    Ok(())
}

#[command]
#[description = "Tests the bots latency"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    println!("{}, {}", msg.timestamp, Utc::now());
    msg.channel_id.say(&ctx.http, "Pong, this message took ".to_owned() + &DateTime::signed_duration_since(Utc::now(), msg.timestamp).num_milliseconds().to_string() + &"ms".to_owned()).await?;
    
    Ok(())
}

#[command]
#[description = "Check if the bot is active"]
async fn active(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "I am here and listening for your commands").await?;
    Ok(())
}

#[command]
#[description = "Shows a changelog"]
async fn changelog(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let f: (&str, &str);
    let version = if args.is_empty() {
        let config = get_config().expect("Error reading bot version from config");
        config.version
    } else {
        args.single::<String>().unwrap()
    };
    match version.as_str() {
        "2.0" => f = ("Version 2.0", "The entire bot was rewritten in Rust, the commands work slightly different but should all produce the same results"),
        _ => f = ("Unknown Version", "This version does not exist. Valid versions are:\n2.0"),
    }
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(Colour(0xC27C0E));
            e.description("To get a changelog from a prior version (back to 2.0) pass the version number as an argument to this command.");
            e.field(f.0, f.1, true);
            e
        });
        m
    }).await?;
    Ok(())
}

#[command]
#[description = "Shows the avatar of a user"]
async fn avatar(ctx: &Context, msg: &Message) -> CommandResult {
    let mut url: String = "Something went wrong, please try again".to_string();
    if msg.mentions.is_empty() {
        match msg.author.avatar_url() {
            Some(avatar) => url = avatar,
            None => url = msg.author.default_avatar_url(),
        }
    } else {
        match msg.mentions[0].avatar_url() {
            Some(avatar) => url = avatar,
            None => url = msg.mentions[0].default_avatar_url(),
        };
    }
    msg.channel_id.say(&ctx.http, url).await?;
    Ok(())
}

#[command]
#[description = "Shows the default avatar of a user"]
async fn default_avatar(ctx: &Context, msg: &Message) -> CommandResult {
    let mut url: String = "Something went wrong, please try again".to_string();
    if msg.mentions.is_empty() {
        url = msg.author.default_avatar_url();
    } else {
        url = msg.mentions[0].default_avatar_url();
    }
    msg.channel_id.say(&ctx.http, url).await?;
    Ok(())
}

#[command]
#[description = "Reads your input in ICAO code"]
async fn code(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut output = "".to_string();
    for c in args.rest().to_uppercase().chars() {
        match c {
            'A' => output.push_str("Alpha"),
            'B' => output.push_str("Bravo"),
            'C' => output.push_str("Charlie"),
            'D' => output.push_str("Delta"),
            'E' => output.push_str("Echo"),
            'F' => output.push_str("Foxtrott"),
            'G' => output.push_str("Golf"),
            'H' => output.push_str("Hotel"),
            'I' => output.push_str("India"),
            'J' => output.push_str("Juliet"),
            'K' => output.push_str("Kilo"),
            'L' => output.push_str("Lima"),
            'M' => output.push_str("Mike"),
            'N' => output.push_str("November"),
            'O' => output.push_str("Oskar"),
            'P' => output.push_str("Papa"),
            'Q' => output.push_str("Quebec"),
            'R' => output.push_str("Romeo"),
            'S' => output.push_str("Sierra"),
            'T' => output.push_str("Tango"),
            'U' => output.push_str("Uniform"),
            'V' => output.push_str("Victor"),
            'W' => output.push_str("Whiskey"),
            'X' => output.push_str("X-Ray"),
            'Y' => output.push_str("Yankee"),
            'Z' => output.push_str("Zulu"),
            '1' => output.push_str("One"),
            '2' => output.push_str("Two"),
            '3' => output.push_str("Three"),
            '4' => output.push_str("Four"),
            '5' => output.push_str("Five"),
            '6' => output.push_str("Six"),
            '7' => output.push_str("Seven"),
            '8' => output.push_str("Eight"),
            '9' => output.push_str("Nine"),
            '0' => output.push_str("Ten"),
            '.' => output.push_str("Stop"),
            _ => output.push(c),
        }
        output.push_str(", ");
    }
    output.pop();
    output.pop();
    msg.channel_id.say(&ctx.http, output).await?;
    Ok(())
}

#[command]
#[description = "Counts from one number to another with x seconds of delay"]
async fn count(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Current indev command (owner only)"]
async fn debug(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
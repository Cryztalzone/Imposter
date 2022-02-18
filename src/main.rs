use std::{
    collections::{HashSet},
    io,
    io::BufReader,
    fs::File,
};

use serenity::{
    prelude::*,
    async_trait,
    client::bridge::gateway::GatewayIntents,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
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
    application_id: u64
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.dnd().await;
        println!("Successfully connected {}", ready.user.name);
    }
}

#[group]
struct General;

#[group]
#[prefixes("text")]
#[description("A group with commands that make the bot repeat text")]
#[default_command(echo)]
#[commands(echo, say, whisper)]
struct Text;

#[group]
#[prefixes("test")]
#[description("A group with commands to test the bot")]
#[default_command(ping)]
#[commands(active, ping)]
struct Test;

#[group]
#[prefixes("util")]
#[description("A group with utility commands")]
#[default_command(changelog)]
#[commands(avatar, changelog, code, count)]
struct Util;

#[group]
#[owners_only]
#[prefixes("debug")]
#[description("Debug commands")]
#[commands(debug)]
struct Debug;

#[help]
#[command_not_found_text = "Konnte '{}' nicht finden."]
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
    msg.channel_id.say(&ctx.http, &args.rest()).await?;
    Ok(())
}

#[command]
#[description = "Repeats your message with TTS"]
async fn say(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Deletes your message, then repeats it with TTS (leaves no trace of the author)"]
async fn whisper(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Tests the bots latency"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Check if the bot is active"]
async fn active(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Shows a changelog"]
async fn changelog(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Shows the avatar of a user"]
async fn avatar(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Reads your input in ICAO code"]
async fn code(ctx: &Context, msg: &Message) -> CommandResult {
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
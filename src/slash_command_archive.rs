use reqwest::Client as ReqwestClient;
use serde_json::{json, Value};

fn main() {
    let http_client = Http::new_with_token_application_id(&config.token, config.application_id);

        let say = ApplicationCommand::create_global_application_command(&http_client, |command| {
            command.name("say")
            .kind(ApplicationCommandType::ChatInput)
            .description("Repeats a message using TTS")
            .create_option(|option| {
                option
                    .name("message")
                    .description("The message to repeat")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            })
        })
        .await;

        let reqwest_client = ReqwestClient::new();

        //commands(&reqwest_client, 791057190839910430, 237500464445390849, &token).await;
}

async fn register_command(client: &ReqwestClient, application_id: u64, guild_id: u64, token: &String, json: Value) {
    if guild_id != 0 {
        let _ = client.post("https://discord.com/api/v8/applications/".to_owned() + &application_id.to_string() + "/guilds/" + &guild_id.to_string() + "/commands")
            .json(&json)
            .header("Authorization", "Bot ".to_string() + &token)
            .send()
            .await
            .expect("Error sending request");
    } else {
        let _ = client.post("https://discord.com/api/v8/applications/".to_owned() + &application_id.to_string() + "/commands")
        .json(&json)
        .header("Authorization", "Bot ".to_string() + &token)
        .send()
        .await
        .expect("Error sending request");
    }

    println!("Request sent");
}

async fn commands(client: &ReqwestClient, application_id: u64, guild_id: u64, token: &String) {
    let avatar = json!({
        "name": "avatar",
        "description": "Shows the avatar of a user",
        "options": [
            {
                "name": "user",
                "description": "The user to show, you if empty",
                "type": 6
            }
        ]
    });
    let delete = json!({
        "name": "delete",
        "description": "Deletes a channel without triggering delete-guard",
        "options": [
            {
                "name": "channel",
                "description": "The channel to delete",
                "type": 7,
                "required": true
            }
        ]
    });
    let changelog = json!({
        "name": "changelog",
        "description": "Shows the changelog"
    });
    let reactionroles = json!({
        "name": "reactionroles",
        "description": "Sends a reactionrole message where users can react to get a role",
        "options": [
            {
                "name": "emoji",
                "description": "The emoji to use",
                "type": 3,
                "required": true
            },
            {
                "name": "role",
                "description": "The role to give with the emoji",
                "type": 8,
                "required": true
            },
            {
                "name": "inactivity",
                "description": "The number of days before the role is automatically removed, leave empty to disable",
                "type": 4
            }
        ]
    });
    let code = json!({
        "name": "code",
        "description": "Reads out a code using TTS",
        "options": [
            {
                "name": "code",
                "description": "The code to read out",
                "type": 3,
                "required": true
            }
        ]
    });
    let count = json!({
        "name": "count",
        "description": "Counts from start to finish in a given interval",
        "options": [
            {
                "name": "start",
                "description": "the number to start counting from",
                "type": 4,
                "required": true
            },
            {
                "name": "end",
                "description": "the number to count to",
                "type": 4,
                "required": true
            },
            {
                "name": "interval",
                "description": "The number of seconds between messages, default 10",
                "type": 4
            }
        ]
    });
    let dick = json!({
        "name": "dick",
        "description": "Try it out",
        "options": [
            {
                "name": "user",
                "description": "You may select a user, you if empty",
                "type": 6
            }
        ]
    });
    let ping = json!({
        "name": "ping",
        "description": "How long the bot takes to respond"
    });
    let say = json!({
        "name": "say",
        "description": "Repeats a message using TTS",
        "options": [
            {
                "name": "message",
                "description": "The message to repeat",
                "type": 3,
                "required": true
            }
        ]
    });

    register_command(&client, application_id, guild_id, &token, avatar).await;
    register_command(&client, application_id, guild_id, &token, delete).await;
    register_command(&client, application_id, guild_id, &token, changelog).await;
    register_command(&client, application_id, guild_id, &token, reactionroles).await;
    register_command(&client, application_id, guild_id, &token, code).await;
    register_command(&client, application_id, guild_id, &token, count).await;
    register_command(&client, application_id, guild_id, &token, dick).await;
    register_command(&client, application_id, guild_id, &token, ping).await;
    register_command(&client, application_id, guild_id, &token, say).await;
}
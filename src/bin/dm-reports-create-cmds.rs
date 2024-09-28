use serde_json::json;
use twilight_http::{request::Request, routing::Route, Client};
use twilight_model::application::command::{Command, CommandType};
use twilight_util::builder::command::CommandBuilder;

#[tokio::main]
async fn main() {
    let discord_token = valk_utils::get_var("DISCORD_TOKEN");
    let client = Client::new(discord_token);
    let cua = client
        .current_user_application()
        .await
        .expect("Failed to get current user")
        .model()
        .await
        .unwrap();
    let discord_server_name = std::env::args().nth(1).expect(
        "This application takes an argument for the name of the server you take DM reports for",
    );
    // This horribleness brought to you by Advaith and Discord's fucking horrendous GA policies
    let command_struct = CommandBuilder::new(
        format!("Report to {}", discord_server_name),
        "",
        CommandType::Message,
    )
    .build();
    let mut command_value = serde_json::to_value(command_struct).unwrap();
    let Some(cmd_value_object) = command_value.as_object_mut() else {
        unreachable!("Serializing a struct and getting not-a-map should be impossible");
    };

    cmd_value_object.insert("integration_types".to_string(), json!([1]));
    cmd_value_object.insert("contexts".to_string(), json!([2]));

    let request = Request::builder(&Route::SetGlobalCommands {
        application_id: cua.id.get(),
    })
    .json(&json!([cmd_value_object]))
    .build()
    .unwrap();
    client.request::<Vec<Command>>(request).await.unwrap();
}

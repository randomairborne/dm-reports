use twilight_http::Client;
use twilight_model::{
    application::{command::CommandType, interaction::InteractionContextType},
    oauth::ApplicationIntegrationType,
};
use twilight_util::builder::command::CommandBuilder;

#[tokio::main(flavor = "current_thread")]
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
    let command = CommandBuilder::new(
        format!("Report to {}", discord_server_name),
        "",
        CommandType::Message,
    )
    .integration_types([ApplicationIntegrationType::UserInstall])
    .contexts([InteractionContextType::PrivateChannel])
    .build();
    client
        .interaction(cua.id)
        .set_global_commands(&[command])
        .await
        .unwrap();
}

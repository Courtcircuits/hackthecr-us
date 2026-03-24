use htc::client::HTCClient;

use crate::{Context, Error};

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Vote for something
///
/// Enter `~vote pumpkin` to vote for pumpkins
#[poise::command(prefix_command, slash_command)]
pub async fn subscribe(
    ctx: Context<'_>,
    #[description = "Subscribe to a restaurant"] choice: String,
) -> Result<(), Error> {
    // Lock the Mutex in a block {} so the Mutex isn't locked across an await point

    let response = format!("Not implemented yet...");
    ctx.say(response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// ~getvotes
/// ~getvotes pumpkin
/// ```
#[poise::command(prefix_command, slash_command)]
pub async fn restaurant(
    ctx: Context<'_>,
    #[description = "Look for a restaurant meal"] restaurant: String,
) -> Result<(), Error> {
    let answer = ctx.data().client.get_restaurants(htc::regions::CrousRegion::Montpellier).await.unwrap();
    ctx.say(format!("Restaurant : {:#?}", answer.get(0).unwrap())).await?;
    Ok(())
}

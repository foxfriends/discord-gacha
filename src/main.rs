use serenity::prelude::*;

mod graphql;
mod shopify;

use shopify::OrderNumber;

struct Data {
    shopify: shopify::Client,
    sheets: sheets::Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn pull(
    ctx: Context<'_>,
    #[description = "Order Number"] order_number: OrderNumber,
) -> Result<(), Error> {
    let order = ctx.data().shopify.get_order(order_number).await?;
    // Confirm number of pulls
    // Save pull state in database (Google Sheet?)
    // Create the Discord post for the pull, with options buttons

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pull()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let sheets = sheets::Client::new(
                    std::env::var("SHEETS_CLIENT_ID").expect("SHEETS_CLIENT_ID is required"),
                    std::env::var("SHEETS_CLIENT_SECRET")
                        .expect("SHEETS_CLIENT_SECRET is required"),
                    std::env::var("SHEETS_REDIRECT_URI").expect("SHEETS_REDIRECT_URI is required"),
                    std::env::var("SHEETS_ACCESS_TOKEN").expect("SHEETS_ACCESS_TOKEN is required"),
                    std::env::var("SHEETS_REFRESH_TOKEN")
                        .expect("SHEETS_REFRESH_TOKEN is required"),
                );

                let shopify = shopify::Client::new(
                    std::env::var("SHOPIFY_SHOP").expect("SHOPIFY_SHOP is required"),
                    std::env::var("SHOPIFY_TOKEN").expect("SHOPIFY_TOKEN is required"),
                );

                Ok(Data { shopify, sheets })
            })
        })
        .build();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is required");
    let client = serenity::Client::builder(token, GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

use a1_notation::Address;
use serenity::prelude::*;
use sheets::types::{DateTimeRenderOption, Dimension, GridProperties, ValueRenderOption};

mod graphql;
mod shopify;

use shopify::OrderNumber;

struct Data {
    shopify: shopify::Client,
    sheets: sheets::Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn sheet_id() -> String {
    std::env::var("SHEETS_SHEET_ID").unwrap()
}

/// Take a pull on Kittyalyst's Fire Emblem Gacha Machine.
///
/// Requires a valid purchase order number from https://kittyalyst.com.
#[poise::command(slash_command, prefix_command)]
async fn pull(
    ctx: Context<'_>,
    #[description = "Order Number"] order_number: OrderNumber,
) -> Result<(), Error> {
    log::info!("User attempting pull: {}", order_number);
    let data = ctx.data();
    let order = data.shopify.get_order(order_number).await?;
    log::debug!("order={:#?}", order);
    let pulls = order.line_items.nodes.len(); // TODO: Check for matching SKUs

    let spreadsheet = data
        .sheets
        .spreadsheets()
        .get(&sheet_id(), true, &[])
        .await?
        .body;
    let grid_properties = spreadsheet.sheets[0]
        .properties
        .as_ref()
        .unwrap()
        .grid_properties
        .as_ref()
        .unwrap();
    let column_headers = data
        .sheets
        .spreadsheets()
        .values_get(
            &sheet_id(),
            &format!(
                "{}:{}",
                Address::new(0, 0),
                Address::new(grid_properties.column_count as usize, 0)
            ),
            DateTimeRenderOption::Noop,
            Dimension::Rows,
            ValueRenderOption::Noop,
        )
        .await?
        .body;

    let order_number_column = column_headers.values[0]
        .iter()
        .position(|x| x == "Order Number")
        .expect("`Order Number` column must exist");
    let discord_user_column = column_headers.values[0]
        .iter()
        .position(|x| x == "Discord User")
        .expect("`Discord User` column must exist");
    let pulls_column = column_headers.values[0]
        .iter()
        .position(|x| x == "Pulls")
        .expect("`Pulls` column must exist");

    // Save pull state in database (Google Sheet?)
    // Create the Discord post for the pull, with options buttons

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let mut sheets = sheets::Client::new(
        std::env::var("SHEETS_CLIENT_ID").expect("SHEETS_CLIENT_ID is required"),
        std::env::var("SHEETS_CLIENT_SECRET").expect("SHEETS_CLIENT_SECRET is required"),
        std::env::var("SHEETS_REDIRECT_URI").expect("SHEETS_REDIRECT_URI is required"),
        std::env::var("SHEETS_ACCESS_TOKEN").expect("SHEETS_ACCESS_TOKEN is required"),
        std::env::var("SHEETS_REFRESH_TOKEN").expect("SHEETS_REFRESH_TOKEN is required"),
    );

    println!(
        "To add this bot to a server:\n\thttps://discord.com/api/oauth2/authorize?client_id={}&permissions2048=&scope=bot%20applications.commands",
        std::env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID is required")
    );

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pull()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

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

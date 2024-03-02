use a1_notation::Address;
use serenity::prelude::*;
use sheets::types::{
    DateTimeRenderOption, Dimension, InsertDataOption, ValueInputOption, ValueRange,
    ValueRenderOption,
};

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
    let Context::Application(ctx) = ctx else {
        unreachable!("Slash command is always this");
    };

    log::info!("User attempting pull: {}", order_number);
    let data = ctx.data();
    let order = data.shopify.get_order(order_number).await?;
    log::debug!("order={:#?}", order);
    let pulls = order.line_items.nodes.len(); // TODO: Check for matching SKUs

    let spreadsheet = data
        .sheets
        .spreadsheets()
        .get(&sheet_id(), false, &[])
        .await?;
    let properties = spreadsheet.body.sheets[0].properties.as_ref().unwrap();
    let grid_properties = properties.grid_properties.as_ref().unwrap();

    let response = data
        .sheets
        .spreadsheets()
        .values_get(
            &sheet_id(),
            &format!("A2:{}", Address::new(0, grid_properties.row_count as usize)),
            DateTimeRenderOption::Noop,
            Dimension::Columns,
            ValueRenderOption::Noop,
        )
        .await?
        .body;
        dbg!(&response);
    let order_numbers = response
        .values
        .first()
        .unwrap_or(&vec![])
        .iter()
        .map(|val| val.parse().ok())
        .collect::<Vec<Option<OrderNumber>>>();
    if !order_numbers.contains(&Some(order_number)) {
        data.sheets
            .spreadsheets()
            .values_append(
                &sheet_id(),
                "A1:D1",
                false,
                InsertDataOption::InsertRows,
                DateTimeRenderOption::Noop,
                ValueRenderOption::Noop,
                ValueInputOption::Raw,
                &ValueRange {
                    major_dimension: Some(Dimension::Rows),
                    range: "A1:D1".to_owned(),
                    values: vec![vec![
                        order_number.to_string(),
                        ctx.interaction.user.id.to_string(),
                        ctx.interaction.user.name.to_string(),
                        pulls.to_string(),
                    ]],
                },
            )
            .await?;
    }

    // Create the Discord post for the pull, with options buttons

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let sheets = sheets::Client::new(
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

use poise::dispatch::FrameworkContext;
use poise::serenity_prelude::{self as serenity, *};
use poise::BoxFuture;
use serde::{Deserialize, Serialize};

mod config;
mod database;
mod error;
mod graphql;
mod shopify;

use config::{Pools, Products};
use database::{PullsData, Row, Sheets};
use error::CustomError;
use shopify::OrderNumber;

struct Data {
    shopify: shopify::Client,
    sheets: Sheets,
    pools: Pools,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
struct InteractionType {
    order_number: OrderNumber,
    action: Action,
}

const PRODUCTS: &str = include_str!("../products.toml");

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum Action {
    Single,
    Bulk,
    Pull(usize),
    Share,
}

/// Take a pull on Kittyalyst's Fire Emblem Gacha Machine.
///
/// Requires a valid purchase order number from https://kittyalyst.com.
#[poise::command(slash_command)]
async fn pull(
    ctx: Context<'_>,
    #[description = "Order Number"] order_number: OrderNumber,
) -> Result<(), Error> {
    let Context::Application(ctx) = ctx else {
        unreachable!("Slash command is always application");
    };

    log::info!("User attempting pull: {}", order_number);
    let data = ctx.data();
    let order = data.shopify.get_order(order_number).await?;
    log::debug!("Pulling for order: {:#?}", order);

    let row = data
        .sheets
        .get_order(order_number)
        .await?
        .unwrap_or_else(|| {
            Row::new(
                order_number,
                ctx.interaction.user.id.to_string(),
                ctx.interaction.user.name.to_owned(),
                PullsData::new(3, 2),
            )
        });
    log::debug!("Pull state: {:#?}", row);

    let message = row.pulls.to_message(order_number)?;
    data.sheets.save(row).await?;
    ctx.send(message.into_reply()).await?;

    Ok(())
}

async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let interaction_id: InteractionType = serde_json::from_str(&interaction.data.custom_id)?;
    log::info!("Received interaction {:?}", interaction_id);
    let mut row = data
        .sheets
        .get_order(interaction_id.order_number)
        .await?
        .ok_or_else(|| CustomError("Pull data for this order could not be found.".to_owned()))?;

    match interaction_id.action {
        Action::Single => row.pulls.start_banner_single()?,
        Action::Bulk => row.pulls.start_banner_bulk()?,
        Action::Pull(index) => {
            let pool = row.pulls.check_slot(index).ok_or_else(|| {
                CustomError("There is no currently active pull".to_owned())
            })?;
            let product = data.pools.pull(pool);
            row.pulls.pull_slot(index, product)?;
        }
        Action::Share => {}
    }

    let message = row.pulls.to_message(row.order_number)?;

    data.sheets.save(row).await?;
    let (response, files) = message.into_interaction_response();
    ctx.http
        .create_interaction_response(
            interaction.id,
            &interaction.token,
            &CreateInteractionResponse::UpdateMessage(response),
            files,
        )
        .await?;
    Ok(())
}

fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a FullEvent,
    _: FrameworkContext<'a, Data, Error>,
    data: &'a Data,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        match event {
            FullEvent::InteractionCreate {
                interaction: Interaction::Component(interaction),
            } => {
                if let Err(error) = handle_interaction(ctx, interaction, data).await {
                    ctx.http
                        .create_interaction_response(
                            interaction.id,
                            &interaction.token,
                            &CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .ephemeral(true)
                                    .content(format!("Error: {error}")),
                            ),
                            vec![],
                        )
                        .await?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    })
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let pools = Products::from_toml(PRODUCTS).unwrap().into_pools();
    println!("{}", pools.distribution());

    println!(
        "To add this bot to a server:\n\thttps://discord.com/api/oauth2/authorize?client_id={}&permissions2048=&scope=bot%20applications.commands",
        std::env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID is required")
    );

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pull()],
            event_handler,
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let sheets = Sheets::new(
                    std::env::var("SHEETS_SHEET_ID").expect("SHEETS_SHEET_ID is required"),
                );
                let shopify = shopify::Client::new(
                    std::env::var("SHOPIFY_SHOP").expect("SHOPIFY_SHOP is required"),
                    std::env::var("SHOPIFY_TOKEN").expect("SHOPIFY_TOKEN is required"),
                );

                Ok(Data {
                    shopify,
                    sheets,
                    pools,
                })
            })
        })
        .build();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is required");
    let client = serenity::Client::builder(token, GatewayIntents::non_privileged())
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

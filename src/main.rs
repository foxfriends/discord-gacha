use poise::dispatch::FrameworkContext;
use poise::serenity_prelude::{self as serenity, *};
use poise::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

mod config;
mod database;
mod error;
mod graphql;
mod inventory;
mod shopify;

use config::Products;
use database::{PullsData, Row, Sheets};
use error::CustomError;
use shopify::OrderNumber;

struct Data {
    shopify: shopify::Client,
    sheets: Sheets,
    products: Products,
    inventory: inventory::Client,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
struct InteractionType {
    order_number: OrderNumber,
    action: Action,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum Action {
    Single,
    Bulk,
    Pull(usize),
    Share,
}

/// Summon enamel pins on Kittyalyst's gacha machine.
///
/// Requires a valid purchase order number from https://www.kittyalyst.com/products/kitty-emblem-gacha
#[poise::command(slash_command, ephemeral)]
async fn summon(
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

    let row = data.sheets.get_order(order_number).await?;
    let row = match row {
        Some(row) => row,
        None => {
            let (singles, bulks) = if std::env::var("PULLS_FREE").is_ok() {
                (3, 2)
            } else {
                let tickets = data
                    .products
                    .ticket
                    .iter()
                    .map(|ticket| (&ticket.sku, (ticket.singles, ticket.bulks)))
                    .collect::<HashMap<_, _>>();

                order
                    .line_items
                    .nodes
                    .iter()
                    .filter_map(|product| {
                        let (single, bulk) = tickets.get(&product.sku)?;
                        Some((single * product.quantity, bulk * product.quantity))
                    })
                    .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
            };

            if singles == 0 && bulks == 0 {
                log::warn!("Trying to pull {} with no purchased tickets", order_number);
                ctx.say("There are no available summons associated with this order. Buy a summon for a valid order number: https://www.kittyalyst.com/products/kitty-emblem-gacha")
                    .await?;
                return Ok(());
            } else {
                Row::new(
                    order_number,
                    ctx.interaction.user.id.to_string(),
                    ctx.interaction.user.name.to_owned(),
                    PullsData::new(singles, bulks),
                )
            }
        }
    };

    log::debug!("Pull state: {:#?}", row);

    if row.discord_user_id != ctx.interaction.user.id.to_string() {
        log::warn!("Wrong user pulled for order {}", order_number);
        ctx.say(
            "This order number has already been summoned for by someone else. Are you sure it's yours?",
        )
        .await?;
        return Ok(());
    }
    let message = row.pulls.to_message(order_number, None)?;
    ctx.send(message.into_reply()).await?;
    data.sheets.save(row).await?;

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

    let mut extra = None;
    match interaction_id.action {
        Action::Single => {
            let inventory = data.inventory.get_inventory().await.map_err(|err| {
                log::error!("Failed to check inventory: {}", err);
                CustomError("Failed to check shop inventory, try again later.".to_owned())
            })?;
            row.pulls.start_banner_single(&data.products, &inventory)?;
        }
        Action::Bulk => {
            let inventory = data.inventory.get_inventory().await.map_err(|err| {
                log::error!("Failed to check inventory: {}", err);
                CustomError("Failed to check shop inventory, try again later.".to_owned())
            })?;
            row.pulls.start_banner_bulk(&data.products, &inventory)?;
        }
        Action::Pull(index) => {
            // NOTE: because we're not checking available inventory at this point, there is potential
            // that two people pull at the same time, queuing up the last of one product, then both
            // people pull that same product and it becomes oversold. Chances are very low though.
            let product = row
                .pulls
                .check_slot(index)
                .ok_or_else(|| CustomError("There is no currently active summon".to_owned()))?;
            if let Err(error) = data
                .inventory
                .log_pull(
                    interaction.user.id.to_string(),
                    interaction_id.order_number,
                    product.sku.clone(),
                )
                .await
            {
                log::error!("Error saving order to inventory: {}", error);
            }

            extra = Some(format!("You got **{}**!", product.name));
            if let Err(error) = row.pulls.pull_slot(index) {
                extra = Some(format!(
                    "An error has occurred, please try again. ({error})"
                ));
            }
        }
        Action::Share => {
            let response = row
                .pulls
                .into_share_message(format!("<@{}>", row.discord_user_id))?;
            ctx.http
                .create_interaction_response(
                    interaction.id,
                    &interaction.token,
                    &CreateInteractionResponse::Acknowledge,
                    vec![],
                )
                .await?;
            log::info!(
                "Sending share to {} ({})",
                interaction.channel_id,
                interaction
                    .channel
                    .as_ref()
                    .and_then(|chan| chan.name.as_deref())
                    .unwrap_or("unknown")
            );
            interaction
                .channel_id
                .send_message(&ctx.http, response)
                .await?;
            return Ok(());
        }
    }

    let message = row.pulls.to_message(row.order_number, extra)?;
    let (response, files) = message.into_interaction_response();
    ctx.http
        .create_interaction_response(
            interaction.id,
            &interaction.token,
            &CreateInteractionResponse::UpdateMessage(response),
            files,
        )
        .await?;
    data.sheets.save(row).await?;
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
            // Direct messages to the bot can be logged
            FullEvent::Message { new_message } if new_message.guild_id.is_none() && !new_message.author.bot => {
                log::debug!("{:#?}", new_message);
                log::info!("{} says: {}", new_message.author.name, new_message.content);
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

    let products = std::fs::read_to_string("./products.toml").expect("products.toml must exist");
    let products = Products::from_toml(&products).unwrap();
    let assets_dir: PathBuf = "./assets/".parse().unwrap();
    for product in products.iter() {
        if !assets_dir.join(&product.sku).with_extension("png").exists() {
            log::warn!("Missing image for {}", product.sku)
        }
    }

    println!("{}", products.distribution());
    println!(
        "To add this bot to a server:\n\thttps://discord.com/api/oauth2/authorize?client_id={}&permissions=274877941760&scope=bot%20applications.commands",
        std::env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID is required")
    );

    if std::env::var("PULLS_FREE").is_ok() {
        log::warn!("Pulls are free!");
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![summon()],
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
                let inventory = inventory::Client::new(
                    &std::env::var("INVENTORY_URL").expect("INVENTORY_URL is required"),
                    std::env::var("INVENTORY_ENABLED").is_ok(),
                );

                Ok(Data {
                    shopify,
                    sheets,
                    products,
                    inventory,
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

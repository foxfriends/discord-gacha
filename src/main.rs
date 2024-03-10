use image::imageops::{overlay, FilterType};
use image::load_from_memory;
use poise::serenity_prelude::*;
use poise::CreateReply;
use serde::{Deserialize, Serialize};

mod google;
mod graphql;
mod shopify;

use google::{Row, Sheets};
use shopify::OrderNumber;

struct Data {
    shopify: shopify::Client,
    sheets: Sheets,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const SUMMON_PNG: &[u8] = include_bytes!("../assets/summon.png");
const ROBIN_PNG: &[u8] = include_bytes!("../assets/Robin.png");

#[derive(Serialize, Deserialize, Debug)]
enum Pool {
    Red,
    Blue,
    Green,
    White,
}

#[derive(Serialize, Deserialize, Debug)]
struct Banner {
    pools: Vec<Pool>,
    pulls: Vec<Option<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PullsData {
    bulks: usize,
    singles: usize,
    bulk_pulls: Vec<Banner>,
    single_pulls: Vec<Banner>,
}

impl PullsData {
    fn new(singles: usize, bulks: usize) -> Self {
        Self {
            singles,
            bulks,
            bulk_pulls: vec![],
            single_pulls: vec![],
        }
    }

    fn skus(&self) -> impl Iterator<Item = String> + '_ {
        self.bulk_pulls
            .iter()
            .chain(self.single_pulls.iter())
            .flat_map(|banner| banner.pulls.iter())
            .flatten()
            .cloned()
    }
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

    let mut database = data.sheets.database().await?;
    let row = database.remove(&order_number).unwrap_or_else(|| {
        Row::new(
            order_number,
            ctx.interaction.user.id.to_string(),
            ctx.interaction.user.name.to_owned(),
            PullsData::new(3, 2),
        )
    });
    log::debug!("Pull state: {:?}", row);

    // Create the Discord post for the pull, with options buttons
    let mut summon = load_from_memory(SUMMON_PNG).unwrap();
    let robin = load_from_memory(ROBIN_PNG)
        .unwrap()
        .resize(180, 180, FilterType::Triangle);
    overlay(&mut summon, &robin, 270, 90);
    let mut bytes: Vec<u8> = Vec::new();
    summon.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageOutputFormat::Png,
    )?;
    let message = CreateReply::default()
        .content("Select a stone. The colors indicate the Hero types.")
        .attachment(CreateAttachment::bytes(bytes, "summon.png"));

    data.sheets.save(row).await?;

    ctx.send(message).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

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

                let sheets = Sheets::new(
                    std::env::var("SHEETS_SHEET_ID").expect("SHEETS_SHEET_ID is required"),
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

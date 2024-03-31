use crate::config::{Banner, Pool, Product};
use crate::shopify::OrderNumber;
use crate::{Action, CustomError, InteractionType};
use poise::serenity_prelude::*;
use poise::CreateReply;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

pub struct Message {
    message: String,
    buttons: Vec<CreateActionRow>,
    image: Option<Vec<u8>>,
}

impl Message {
    pub fn into_interaction_response(
        self,
    ) -> (CreateInteractionResponseMessage, Vec<CreateAttachment>) {
        let response = CreateInteractionResponseMessage::new()
            .content(self.message)
            .components(self.buttons)
            .ephemeral(true);

        if let Some(image) = self.image {
            let file_placeholder = CreateAttachment::bytes(vec![], "summon.png");
            let file = CreateAttachment::bytes(image, "summon.png");
            (response.add_file(file_placeholder), vec![file])
        } else {
            (response, vec![])
        }
    }

    pub fn into_reply(self) -> CreateReply {
        let response = CreateReply::default()
            .content(self.message)
            .components(self.buttons)
            .ephemeral(true);

        if let Some(image) = self.image {
            response.attachment(CreateAttachment::bytes(image, "summon.png"))
        } else {
            response
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActiveBanner {
    Single(Banner),
    Bulk(Banner),
    None,
}

impl ActiveBanner {
    fn as_banner(&self) -> Option<&Banner> {
        match &self {
            Self::Single(banner) | Self::Bulk(banner) => Some(banner),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PullsData {
    pub bulks: usize,
    pub singles: usize,
    pub bulk_pulls: Vec<Banner>,
    pub single_pulls: Vec<Banner>,
    pub active: ActiveBanner,
}

impl PullsData {
    pub fn new(singles: usize, bulks: usize) -> Self {
        Self {
            singles,
            bulks,
            bulk_pulls: vec![],
            single_pulls: vec![],
            active: ActiveBanner::None,
        }
    }

    #[allow(dead_code)]
    pub fn skus(&self) -> impl Iterator<Item = String> + '_ {
        self.bulk_pulls
            .iter()
            .chain(self.single_pulls.iter())
            .chain(self.active.as_banner())
            .flat_map(|banner| banner.pulls.iter())
            .flatten()
            .map(|product| product.sku.to_owned())
    }

    pub fn names(&self) -> impl Iterator<Item = String> + '_ {
        self.bulk_pulls
            .iter()
            .chain(self.single_pulls.iter())
            .chain(self.active.as_banner())
            .flat_map(|banner| banner.pulls.iter())
            .flatten()
            .map(|product| product.name.to_owned())
    }

    fn pulled_singles(&self) -> usize {
        let past: usize = self.single_pulls.iter().map(|pull| pull.pulled()).sum();
        let active = match &self.active {
            ActiveBanner::Single(banner) => banner.pulled(),
            _ => 0,
        };
        past + active
    }

    fn pulled_bulks(&self) -> usize {
        let past = self.bulk_pulls.len();
        if matches!(self.active, ActiveBanner::Bulk(..)) {
            past + 1
        } else {
            past
        }
    }

    pub fn to_message(&self, order_number: OrderNumber) -> Result<Message, crate::Error> {
        let singles_available = self.singles - self.pulled_singles();
        let bulks_available = self.bulks - self.pulled_bulks();

        let start_banner = match &self.active {
            ActiveBanner::None => true,
            ActiveBanner::Single(banner) if banner.pulled() > 0 => true,
            ActiveBanner::Bulk(banner) if banner.pulled() == 5 => true,
            _ => false,
        };

        let mut buttons = vec![];

        let continue_banner = match &self.active {
            ActiveBanner::Single(banner) if banner.pulled() != 5 && singles_available > 0 => {
                Some(banner)
            }
            ActiveBanner::Bulk(banner) if banner.pulled() != 5 => Some(banner),
            _ => None,
        };
        if let Some(banner) = continue_banner {
            let mut row = vec![];
            for (i, pool) in banner.pools.iter().enumerate() {
                if banner.pulls[i].is_none() {
                    row.push(
                        CreateButton::new(
                            serde_json::to_string(&InteractionType {
                                order_number,
                                action: Action::Pull(i),
                            })
                            .unwrap(),
                        )
                        .label(format!("Summon {pool} #{}", i + 1))
                        .style(match pool {
                            Pool::Red => ButtonStyle::Danger,
                            Pool::Green => ButtonStyle::Success,
                            Pool::Blue => ButtonStyle::Primary,
                            Pool::White => ButtonStyle::Secondary,
                        }),
                    );
                }
            }
            buttons.push(CreateActionRow::Buttons(row));
        }

        let mut row = vec![];
        if start_banner {
            if singles_available > 0 {
                row.push(
                    CreateButton::new(
                        serde_json::to_string(&InteractionType {
                            order_number,
                            action: Action::Single,
                        })
                        .unwrap(),
                    )
                    .label("Start new single summon"),
                );
            }

            if bulks_available > 0 {
                row.push(
                    CreateButton::new(
                        serde_json::to_string(&InteractionType {
                            order_number,
                            action: Action::Bulk,
                        })
                        .unwrap(),
                    )
                    .label("Start new full summon"),
                );
            }
        }
        if !matches!(self.active, ActiveBanner::None) {
            row.push(
                CreateButton::new(
                    serde_json::to_string(&InteractionType {
                        order_number,
                        action: Action::Share,
                    })
                    .unwrap(),
                )
                .label("Share current results")
                .style(ButtonStyle::Secondary),
            );
        }
        if !row.is_empty() {
            buttons.push(CreateActionRow::Buttons(row));
        }

        let image = match &self.active {
            ActiveBanner::Single(banner) | ActiveBanner::Bulk(banner) => Some(banner.to_image()?),
            _ => None,
        };

        let mut message = String::new();
        writeln!(&mut message, "You are viewing **Order {order_number}**.")?;
        writeln!(
            &mut message,
            "You have **{} full summons** and **{} single summons** remaining.",
            bulks_available, singles_available
        )?;

        if continue_banner.is_some() && start_banner {
            writeln!(&mut message, "You may continue making single summons from the current pool or choose to start a new one.")?;
        }

        if continue_banner.is_some() {
            writeln!(
                &mut message,
                "You have started a full summon. Choose an option to continue."
            )?;
        }

        if start_banner && matches!(self.active, ActiveBanner::None) {
            writeln!(&mut message, "Choose an option to begin summoning.")?;
        } else if start_banner {
            writeln!(
                &mut message,
                "Choose an option below to continue summoning."
            )?;
        }

        Ok(Message {
            message,
            buttons,
            image,
        })
    }

    pub fn into_share_message(
        self,
        discord_reference: String,
    ) -> Result<CreateMessage, crate::Error> {
        let response =
            CreateMessage::new().content(format!("{discord_reference} has shared their pull!"));

        match self.active.as_banner() {
            Some(banner) => {
                let image = banner.to_image()?;
                let file = CreateAttachment::bytes(image, "summon.png");
                Ok(response.add_file(file))
            }
            _ => Ok(response),
        }
    }

    pub fn pull_slot(&mut self, slot: usize, pull: &Product) -> Result<(), CustomError> {
        let pulled_singles = self.pulled_singles();
        match &mut self.active {
            ActiveBanner::Single(..) if pulled_singles >= self.singles => {
                return Err(CustomError(
                    "There are no more single summons available for this order.".to_owned(),
                ));
            }
            ActiveBanner::Single(banner) | ActiveBanner::Bulk(banner) if banner.pulled() == 5 => {
                return Err(CustomError(
                    "This pool is empty. Start a new one.".to_owned(),
                ))
            }
            ActiveBanner::Single(banner) | ActiveBanner::Bulk(banner)
                if banner.pulls[slot].is_some() =>
            {
                return Err(CustomError(
                    "This hero has already been summoned.".to_owned(),
                ))
            }
            ActiveBanner::Single(banner) | ActiveBanner::Bulk(banner) => {
                banner.pulls[slot] = Some(pull.clone());
            }
            ActiveBanner::None => {
                return Err(CustomError(
                    "There is not currently an active pool for this order.".to_owned(),
                ));
            }
        }
        Ok(())
    }

    pub fn check_slot(&self, slot: usize) -> Option<Pool> {
        Some(self.active.as_banner()?.pools[slot])
    }

    pub fn start_banner_single(&mut self) -> Result<(), CustomError> {
        let pulls_remaining = self.singles - self.pulled_singles();
        if pulls_remaining == 0 {
            return Err(CustomError(
                "There are no more single summons available for this order.".to_owned(),
            ));
        }
        match &self.active {
            ActiveBanner::Single(banner) if banner.pulled() == 0 => {
                return Err(CustomError("You may not reroll a fresh banner.".to_owned()));
            }
            ActiveBanner::Bulk(banner) if banner.pulled() != 5 => {
                return Err(CustomError(
                    "There is a bulk pull in progress already, which must be completed first."
                        .to_owned(),
                ));
            }
            _ => self.set_active(ActiveBanner::Single(Banner::default())),
        }
        Ok(())
    }

    pub fn start_banner_bulk(&mut self) -> Result<(), CustomError> {
        let pulls_remaining = self.bulks - self.bulk_pulls.len();
        if pulls_remaining == 0 {
            return Err(CustomError(
                "There are no more bulk pulls available for this order".to_owned(),
            ));
        }
        match &self.active {
            ActiveBanner::Single(banner) if banner.pulled() == 0 => {
                return Err(CustomError(
                    "There is a single pull in progress already, which must be completed first"
                        .to_owned(),
                ));
            }
            ActiveBanner::Bulk(banner) if banner.pulled() != 5 => {
                return Err(CustomError(
                    "There is a bulk pull in progress already, which must be completed first"
                        .to_owned(),
                ));
            }
            _ => self.set_active(ActiveBanner::Bulk(Banner::default())),
        }
        Ok(())
    }

    fn set_active(&mut self, banner: ActiveBanner) {
        match std::mem::replace(&mut self.active, banner) {
            ActiveBanner::Single(banner) => self.single_pulls.push(banner),
            ActiveBanner::Bulk(banner) => self.bulk_pulls.push(banner),
            ActiveBanner::None => {}
        }
    }
}

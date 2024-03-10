use a1_notation::Address;
use sheets::types::{
    DateTimeRenderOption, Dimension, InsertDataOption, ValueInputOption, ValueRange,
    ValueRenderOption,
};
use std::collections::HashMap;

use crate::shopify::OrderNumber;
use crate::PullsData;

pub struct Sheets {
    client: sheets::Client,
    sheet_id: String,
}

impl Sheets {
    pub fn new(sheet_id: String) -> Self {
        let client = sheets::Client::new(
            std::env::var("SHEETS_CLIENT_ID").expect("SHEETS_CLIENT_ID is required"),
            std::env::var("SHEETS_CLIENT_SECRET").expect("SHEETS_CLIENT_SECRET is required"),
            std::env::var("SHEETS_REDIRECT_URI").expect("SHEETS_REDIRECT_URI is required"),
            std::env::var("SHEETS_ACCESS_TOKEN").expect("SHEETS_ACCESS_TOKEN is required"),
            std::env::var("SHEETS_REFRESH_TOKEN").expect("SHEETS_REFRESH_TOKEN is required"),
        );
        Self { client, sheet_id }
    }

    pub async fn database(&self) -> Result<HashMap<OrderNumber, Row>, sheets::ClientError> {
        if self.client.is_expired().await != Some(false) {
            self.client.refresh_access_token().await?;
        }
        let spreadsheet = self
            .client
            .spreadsheets()
            .get(&self.sheet_id, false, &[])
            .await?;
        let properties = spreadsheet.body.sheets[0].properties.as_ref().unwrap();
        let grid_properties = properties.grid_properties.as_ref().unwrap();
        let response = self
            .client
            .spreadsheets()
            .values_get(
                &self.sheet_id,
                &format!("A1:{}", Address::new(3, grid_properties.row_count as usize)),
                DateTimeRenderOption::Noop,
                Dimension::Rows,
                ValueRenderOption::Noop,
            )
            .await?
            .body;
        let rows = response
            .values
            .into_iter()
            .enumerate()
            .filter_map(|(i, row)| {
                let mut row = Row::try_from(row).ok()?;
                row.existing = Some(i);
                Some(row)
            })
            .collect::<Vec<Row>>();

        log::debug!("{} rows loaded", rows.len());

        Ok(rows
            .into_iter()
            .map(|row| (row.order_number, row))
            .collect())
    }

    pub async fn save(&self, row: Row) -> Result<(), sheets::ClientError> {
        let row_index = row.existing;
        let values = row.into_cells();
        let range = format!(
            "{}:{}",
            Address::new(0, row_index.unwrap_or(0)),
            Address::new(values.len() - 1, row_index.unwrap_or(0))
        );
        if row_index.is_some() {
            self.client
                .spreadsheets()
                .values_update(
                    &self.sheet_id,
                    &range,
                    false,
                    DateTimeRenderOption::Noop,
                    ValueRenderOption::Noop,
                    ValueInputOption::Raw,
                    &ValueRange {
                        major_dimension: Some(Dimension::Rows),
                        range: range.to_owned(),
                        values: vec![values],
                    },
                )
                .await?;
        } else {
            self.client
                .spreadsheets()
                .values_append(
                    &self.sheet_id,
                    &range,
                    false,
                    InsertDataOption::InsertRows,
                    DateTimeRenderOption::Noop,
                    ValueRenderOption::Noop,
                    ValueInputOption::Raw,
                    &ValueRange {
                        major_dimension: Some(Dimension::Rows),
                        range: range.to_owned(),
                        values: vec![values],
                    },
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Row {
    pub order_number: OrderNumber,
    pub discord_user_id: String,
    pub discord_username: String,
    pub pulls: PullsData,

    existing: Option<usize>,
}

impl Row {
    pub fn new(
        order_number: OrderNumber,
        discord_user_id: String,
        discord_username: String,
        pulls: PullsData,
    ) -> Self {
        Self {
            order_number,
            discord_user_id,
            discord_username,
            pulls,
            existing: None,
        }
    }

    fn into_cells(self) -> Vec<String> {
        let mut values = vec![
            self.order_number.to_string(),
            self.discord_user_id,
            self.discord_username,
            serde_json::to_string(&self.pulls).unwrap(),
        ];
        values.extend(self.pulls.skus());
        values
    }
}

impl TryFrom<Vec<String>> for Row {
    type Error = ();

    fn try_from(row: Vec<String>) -> Result<Self, Self::Error> {
        let mut row = row.into_iter();
        Ok(Self {
            order_number: row.next().ok_or(())?.parse().map_err(|_| ())?,
            discord_user_id: row.next().ok_or(())?,
            discord_username: row.next().ok_or(())?,
            pulls: serde_json::from_str(&row.next().ok_or(())?).map_err(|_| ())?,

            existing: None,
        })
    }
}

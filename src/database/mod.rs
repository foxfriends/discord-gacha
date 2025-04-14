use a1_notation::Address;
use sheets::types::{
    DateTimeRenderOption, Dimension, InsertDataOption, ValueInputOption, ValueRange,
    ValueRenderOption,
};
use std::collections::HashMap;
use std::io::Write;

mod pulls_data;
mod row;

use crate::shopify::OrderNumber;
pub use pulls_data::PullsData;
pub use row::Row;

#[derive(Clone)]
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

    pub async fn get_consent(&mut self) -> Result<(), String> {
        match self.client.refresh_access_token().await {
            Ok(token) if token.access_token.is_empty() => {}
            Ok(..) => return Ok(()),
            Err(error) => {
                log::error!(
                    "Failed to get initial Google Sheets access token: {}",
                    error
                );
            }
        }
        let url = self
            .client
            .user_consent_url(&["https://www.googleapis.com/auth/spreadsheets".to_owned()]);
        println!("Please authorize Google sheets access: {}", url);
        let mut url = String::new();
        print!("Redirected to URL: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut url).unwrap();
        let url: reqwest::Url = url.parse().unwrap();
        let qs = url.query_pairs().collect::<HashMap<_, _>>();
        let code = qs.get("code").unwrap();
        let state = qs.get("state").unwrap();
        let access_token = self
            .client
            .get_access_token(code, state)
            .await
            .map_err(|err| err.to_string())?;
        if access_token.access_token.is_empty() {
            return Err("Did not get a valid access token".to_owned());
        }
        println!("SHEETS_ACCESS_TOKEN={}", access_token.access_token);
        println!("SHEETS_REFRESH_TOKEN={}", access_token.refresh_token);
        Ok(())
    }

    pub async fn database(&self) -> Result<HashMap<OrderNumber, Row>, sheets::ClientError> {
        if self.client.is_expired().await != Some(false) {
            log::debug!("Refreshing access token");
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

    pub async fn get_order(
        &self,
        order_number: OrderNumber,
    ) -> Result<Option<Row>, sheets::ClientError> {
        let mut database = self.database().await?;
        Ok(database.remove(&order_number))
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

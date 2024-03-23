use super::PullsData;
use crate::shopify::OrderNumber;

#[derive(Debug)]
pub struct Row {
    pub order_number: OrderNumber,
    pub discord_user_id: String,
    pub discord_username: String,
    pub pulls: PullsData,

    pub(super) existing: Option<usize>,
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

    pub(super) fn into_cells(self) -> Vec<String> {
        let mut values = vec![
            self.order_number.to_string(),
            self.discord_user_id,
            self.discord_username,
            serde_json::to_string(&self.pulls).unwrap(),
        ];
        values.extend(self.pulls.names());
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

use serde::{Deserialize, Serialize};

use crate::ToSql;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
}

impl Pagination {
    pub const fn page_count(limit: u32, total: u32) -> u32 {
        (total + limit - 1) / limit
    }
}

impl ToSql for Pagination {
    fn to_sql(&self) -> String {
        format!(
            "LIMIT {} OFFSET {}",
            self.limit,
            (self.page - 1) * self.limit
        )
    }
}

impl ToSql for Option<Pagination> {
    fn to_sql(&self) -> String {
        match self {
            Some(pagination) => pagination.to_sql(),
            None => String::new(),
        }
    }
}

pub fn deserialize_option_pagination<'de, D>(
    deserializer: D,
) -> Result<Option<Pagination>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct PaginationFields {
        page: Option<u32>,
        limit: Option<u32>,
    }

    let fields = PaginationFields::deserialize(deserializer)?;

    match (fields.page, fields.limit) {
        (Some(page), Some(limit)) => Ok(Some(Pagination { page, limit })),
        (Some(_), None) => Err(serde::de::Error::missing_field("limit")),
        (None, Some(_)) => Err(serde::de::Error::missing_field("page")),
        (None, None) => Ok(None),
    }
}

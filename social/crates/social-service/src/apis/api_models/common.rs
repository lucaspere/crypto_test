use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::sorts::SortDirection;

/// Common pagination parameters for list endpoints
#[derive(Debug, Deserialize, Default, ToSchema, IntoParams)]
pub struct PaginationOptions {
    /// Page number (starts at 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Number of items per page
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Whether to return all items without pagination
    #[serde(default)]
    pub fetch_all: bool,
}

/// Common sorting parameters for list endpoints
#[derive(Debug, Deserialize, Default, ToSchema)]

pub struct SortOptions<T> {
    /// Field to sort by
    pub sort_by: Option<T>,
    /// Sort direction ("asc" or "desc")
    pub sort_direction: Option<SortDirection>,
}

/// Standard paginated response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// List of items for the current page
    pub items: T,
    /// Total number of items across all pages
    pub total_items: i64,
    /// Current page number
    pub current_page: u32,
    /// Items per page
    pub items_per_page: u32,
    /// Total number of pages
    pub total_pages: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    10
}

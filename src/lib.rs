const GET_PRODUCT_ENDPOINT: &'static str =
    "https://api-extern.systembolaget.se/product/v1/product/";
const GET_ALL_PRODUCTS_ENDPOINT: &'static str =
    "https://api-extern.systembolaget.se/product/v1/product";
const GET_PRODUCTS_WITH_STORE_ENDPOINT: &'static str =
    "https://api-extern.systembolaget.se/product/v1/getproductswithstore";
const SEARCH_ENDPOINT: &'static str = "https://api-extern.systembolaget.se/product/v1/search";
const API_KEY_HEADER: &'static str = "Ocp-Apim-Subscription-Key";

use chrono::{Date, Utc};
use reqwest::{header::HeaderMap, Client};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Serialize_repr, Deserialize_repr};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parse {
        err: serde_json::Error,
        body: String,
    },
    Api(Vec<ApiError>),
    Reqwest(reqwest::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Parse {
            err,
            body: "<unknown request body>".to_string(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<Vec<ApiError>> for Error {
    fn from(err: Vec<ApiError>) -> Error {
        Error::Api(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Parse { err, body } => write!(
                f,
                "Serialization/deserialization error: {}. Response body: '{}'",
                err, body
            ),
            Error::Reqwest(err) => write!(f, "Network error: {}", err),
            Error::Api(errors) => match errors.len() {
                0 => write!(
                    f,
                    "API error: Got error response from API, but no error code or message"
                ),
                1 => write!(f, "API error: {}", errors[0]),
                _ => {
                    let message = errors
                        .iter()
                        .map(|error| format!("({})", error))
                        .collect::<Vec<String>>()
                        .join(", ");
                    write!(f, "API errors: {}", message)
                }
            },
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Parse { err, body: _ } => Some(err),
            Error::Reqwest(err) => Some(err),
            Error::Api(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Systemet {
    client: Client,
}

impl Systemet {
    pub fn new(api_key: String) -> Systemet {
        let mut headers = HeaderMap::new();
        let key = api_key.parse().unwrap();

        headers.insert(API_KEY_HEADER, key);
        let client = Client::builder().default_headers(headers).build().unwrap();
        Systemet { client }
    }

    pub async fn get_product(&self, id: String) -> Result<Product, Error> {
        let url = format!("{}{}", GET_PRODUCT_ENDPOINT, id);
        self.send_request(&url).await
    }

    // TODO: Return an iterator
    pub async fn get_all_products(&self) -> Result<Vec<Product>, Error> {
        self.send_request(GET_ALL_PRODUCTS_ENDPOINT).await
    }

    pub async fn get_products_with_store(&self) -> Result<Vec<ProductsWithStore>, Error> {
        self.send_request(GET_PRODUCTS_WITH_STORE_ENDPOINT).await
    }

    async fn send_request<'de, T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let body = self.client.get(url).send().await?.text().await?;
        match serde_json::from_str::<T>(&body) {
            Ok(product) => Ok(product),
            // Try to parse the body as an error.
            Err(_) => match serde_json::from_str::<Vec<ApiError>>(&body) {
                Ok(api_error) => Err(Error::Api(api_error)),
                Err(err) => Err(Error::Parse { err, body }),
            },
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(i32)]
pub enum SortDirection {
    Ascending = 0,
    Descending = 1,
}

// impl From<&SortDirection> for i32 {
//     fn from(direction: &SortDirection) -> i32 {
//         match direction {
//             SortDirection::Ascending => 0,
//             SortDirection::Descending => 1,
//         }
//     }
// }


#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(i32)]
pub enum SortKey {
    Price = 0,
    Name = 1,
    Volume = 2,
    Vintage = 3,
    Rank = 4,
    City = 5,
    SellStartDate = 6,
    DisplayName = 7,
}

// impl From<&SortKey> for i32 {
//     fn from(key: &SortKey) -> i32 {
//         match key {
//             SortKey::Zero => 0,
//             SortKey::One => 1,
//             SortKey::Two => 2,
//             SortKey::Three => 3,
//             SortKey::Four => 4,
//             SortKey::Five => 5,
//             SortKey::Six => 6,
//             SortKey::Seven => 7,
//         }
//     }
// }

impl SearchRequest {
    pub fn new() -> SearchRequest {
        SearchRequest::default()
    }

    fn validate(&self) -> bool {
        self != &Self::default()
    }

    pub fn alcohol_percentage_max(mut self, alcohol_percentage_max: Option<f64>) -> SearchRequest {
        self.alcohol_percentage_max = alcohol_percentage_max;
        self
    }

    pub fn alcohol_percentage_min(mut self, alcohol_percentage_min: Option<f64>) -> SearchRequest {
        self.alcohol_percentage_min = alcohol_percentage_min;
        self
    }

    pub fn assortment_text(mut self, assortment_text: Option<String>) -> SearchRequest {
        self.assortment_text = assortment_text;
        self
    }

    pub fn bottle_type_group(mut self, bottle_type_group: Option<String>) -> SearchRequest {
        self.bottle_type_group = bottle_type_group;
        self
    }

    pub fn country(mut self, country: Option<String>) -> SearchRequest {
        self.country = country;
        self
    }

    pub fn csr(mut self, csr: Option<String>) -> SearchRequest {
        self.csr = csr;
        self
    }

    /// This property is called "type" in the API) -> SearchRequest { } but that's a keyword in Rust.
    pub fn kind(mut self, kind: Option<String>) -> SearchRequest {
        self.kind = kind;
        self
    }

    pub fn news(mut self, news: Option<String>) -> SearchRequest {
        self.news = news;
        self
    }

    pub fn origin_level_1(mut self, origin_level_1: Option<String>) -> SearchRequest {
        self.origin_level_1 = origin_level_1;
        self
    }

    pub fn origin_level_2(mut self, origin_level_2: Option<String>) -> SearchRequest {
        self.origin_level_2 = origin_level_2;
        self
    }

    pub fn other_selections(mut self, other_selections: Option<String>) -> SearchRequest {
        self.other_selections = other_selections;
        self
    }

    pub fn page(mut self, page: Option<i32>) -> SearchRequest {
        self.page = page;
        self
    }

    pub fn price_max(mut self, price_max: Option<f64>) -> SearchRequest {
        self.price_max = price_max;
        self
    }

    pub fn price_min(mut self, price_min: Option<f64>) -> SearchRequest {
        self.price_min = price_min;
        self
    }

    pub fn seal(mut self, seal: Option<String>) -> SearchRequest {
        self.seal = seal;
        self
    }

    pub fn search_query(mut self, search_query: Option<String>) -> SearchRequest {
        self.search_query = search_query;
        self
    }

    pub fn sell_start_date_from(mut self, sell_start_date_from: Option<Date<Utc>>) -> SearchRequest {
        self.sell_start_date_from = sell_start_date_from;
        self
    }

    pub fn sell_start_date_to(mut self, sell_start_date_to: Option<Date<Utc>>) -> SearchRequest {
        self.sell_start_date_to = sell_start_date_to;
        self
    }

    pub fn sort_by(mut self, sort_by: Option<SortKey>) -> SearchRequest {
        self.sort_by = sort_by;
        self
    }

    pub fn sort_direction(mut self, sort_direction: Option<SortDirection>) -> SearchRequest {
        self.sort_direction = sort_direction;
        self
    }

    pub fn style(mut self, style: Option<String>) -> SearchRequest {
        self.style = style;
        self
    }

    pub fn sub_category(mut self, sub_category: Option<String>) -> SearchRequest {
        self.sub_category = sub_category;
        self
    }

    pub fn vintage(mut self, vintage: Option<String>) -> SearchRequest {
        self.vintage = vintage;
        self
    }
}

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct SearchRequest {
    pub alcohol_percentage_max: Option<f64>,
    pub alcohol_percentage_min: Option<f64>,
    pub assortment_text: Option<String>,
    pub bottle_type_group: Option<String>,
    pub country: Option<String>,
    pub csr: Option<String>,
    /// This property is called "type" in the API, but that's a keyword in Rust.
    pub kind: Option<String>,
    pub news: Option<String>,
    pub origin_level_1: Option<String>,
    pub origin_level_2: Option<String>,
    pub other_selections: Option<String>,
    pub page: Option<i32>,
    pub price_max: Option<f64>,
    pub price_min: Option<f64>,
    pub seal: Option<String>,
    pub search_query: Option<String>,
    #[serde(with = "option_date_serializer")]
    #[serde(default)]
    pub sell_start_date_from: Option<Date<Utc>>,
    #[serde(with = "option_date_serializer")]
    #[serde(default)]
    pub sell_start_date_to: Option<Date<Utc>>,
    pub sort_by: Option<SortKey>,
    pub sort_direction: Option<SortDirection>,
    pub style: Option<String>,
    pub sub_category: Option<String>,
    pub vintage: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Product {
    pub alcohol_percentage: f64,
    pub assortment: Option<String>,
    pub assortment_text: Option<String>,
    pub beverage_description_short: Option<String>,
    pub bottle_text_short: Option<String>,
    pub category: Option<String>,
    pub country: Option<String>,
    pub ethical_label: Option<String>,
    pub is_completely_out_of_stock: bool,
    pub is_ethical: bool,
    pub is_in_store_search_assortment: Option<String>,
    pub is_kosher: bool,
    pub is_manufacturing_country: bool,
    pub is_news: bool,
    pub is_organic: bool,
    pub is_regional_restricted: bool,
    pub is_temporarily_out_of_stock: Option<bool>,
    pub is_web_launch: bool,
    /// This property is called "type" in the API, but that's is a keyword in Rust.
    #[serde(rename = "Type")]
    pub kind: Option<String>,
    pub origin_level_1: Option<String>,
    pub origin_level_2: Option<String>,
    pub price: f64,
    pub producer_name: Option<String>,
    pub product_id: String,
    pub product_name_bold: String,
    pub product_name_thin: Option<String>,
    pub product_number_short: Option<String>,
    pub product_number: String,
    pub recycle_fee: f64,
    pub restricted_parcel_quantity: i32,
    pub seal: Option<String>,
    #[serde(with = "date_serializer")]
    pub sell_start_date: Date<Utc>,
    pub style: Option<String>,
    pub sub_category: Option<String>,
    pub supplier_name: Option<String>,
    pub taste: Option<String>,
    pub usage: Option<String>,
    pub vintage: i32,
    pub volume: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ApiError {
    error: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProductsWithStore {
    site_id: String,
    products: Vec<ProductWithStore>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProductWithStore {
    product_id: String,
    product_number: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error code: {}, message: {}", self.error, self.message)
    }
}

/// Custom serializer for the date format that Systembolaget uses.
/// Stolen from https://serde.rs/custom-date-format.html
mod date_serializer {
    use chrono::{Date, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S>(date: &Date<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.and_hms(0, 0, 0).format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map(|d| d.date())
            .map_err(serde::de::Error::custom)
    }
}

/// Custom serializer for the date format that Systembolaget uses.
/// Stolen from https://serde.rs/custom-date-format.html
mod option_date_serializer {
    use chrono::{Date, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S>(date: &Option<Date<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s;
        if let Some(date) = date {
            s = format!("{}", date.and_hms(0, 0, 0).format(FORMAT));
        } else {
            s = "null".to_string();
        }
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map(|d| d.date())
            .map_err(serde::de::Error::custom)
    }
}

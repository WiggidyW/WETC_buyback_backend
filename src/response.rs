use crate::{
    pricing::Price,
    error::Error,
    item::Item,
};

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::{self, Write},
};

use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub accepted: Vec<AcceptedResultItem>,
    pub rejected: Vec<Item>,
    pub hash: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedResultItem {
    #[serde(flatten)]
    pub item: Item,
    pub price_per: f64,
    pub price_total: f64,
}

impl Response {
    pub fn with_capacity(capacity: usize, location: String) -> Self {
        Response {
            accepted: Vec::with_capacity(capacity),
            rejected: Vec::with_capacity(capacity),
            hash: String::new(),
            location: location,
        }
    }

    pub fn push(&mut self, item: Item, price: Price) {
        match price {
            Price::Accepted(f) => self.accepted.push((item, f).into()),
            Price::Rejected => self.rejected.push(item),
        }
    }

    pub fn to_stdout(&self) -> Result<(), Error> {
        let output: String = self.to_json()?;
        io::stdout()
            .write_all(output.as_ref())
            .map_err(|e| Error::StdoutError(e))
    }

    pub fn with_hash_key(&mut self) -> &str {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        self.hash = format!("{:x}", hasher.finish());
        &self.hash
    }

    pub fn sort(&mut self) {
        self.accepted.sort_by(|a, b| a
            .item
            .name
            .as_str()
            .cmp(b.item.name.as_str()));
        self.rejected.sort_by(|a, b| a
            .name
            .as_str()
            .cmp(b.name.as_str()));
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self)
            .map_err(|e| Error::SerializationError(e))
    }
}

impl Hash for Response {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
        for item in self
            .accepted
            .iter()
        {
            item.item.name.hash(state);
            to_fstring(&item.item.quantity).hash(state);
            to_fstring(&item.price_per).hash(state);
            to_fstring(&item.price_total).hash(state);
        }
        for item in self
            .rejected
            .iter()
        {
            item.name.hash(state);
            to_fstring(&item.quantity).hash(state);
        }
    }
}

impl From<(Item, f64)> for AcceptedResultItem {
    fn from(value: (Item, f64)) -> Self {
        AcceptedResultItem {
            price_per: value.1,
            price_total: value.0.quantity * value.1,
            item: value.0,
        }
    }
}

fn to_fstring(f: &f64) -> String {
    format!("{}", f)
}

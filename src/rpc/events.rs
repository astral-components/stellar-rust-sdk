//! `getEvents` — Soroban contract event stream.

use crate::errors::SdkError;
use crate::rpc::client::RpcClient;
use serde::{Deserialize, Serialize};

/// Topic / contract filter for `getEvents`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventFilter {
    /// `contract`, `system`, or `diagnostic`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// Contract ids (`C…`) to match.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contract_ids: Vec<String>,
    /// Topic filters (each inner vec is an AND of base64 ScVal XDR topics).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<Vec<String>>,
}

impl EventFilter {
    /// Filter for contract events from `contract_id`.
    pub fn contract(contract_id: impl Into<String>) -> Self {
        Self {
            event_type: Some("contract".into()),
            contract_ids: vec![contract_id.into()],
            topics: Vec::new(),
        }
    }

    /// Add a topic matcher (base64 XDR of `ScVal`).
    pub fn with_topic(mut self, topic_xdr_b64: impl Into<String>) -> Self {
        self.topics.push(vec![topic_xdr_b64.into()]);
        self
    }
}

/// A single event from `getEvents`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Event type.
    #[serde(rename = "type", default)]
    pub event_type: Option<String>,
    /// Ledger sequence.
    #[serde(default)]
    pub ledger: Option<u32>,
    /// Ledger close time.
    #[serde(default)]
    pub ledger_closed_at: Option<String>,
    /// Contract id.
    #[serde(default)]
    pub contract_id: Option<String>,
    /// Topic list (base64 XDR).
    #[serde(default)]
    pub topic: Option<Vec<String>>,
    /// Value (base64 XDR).
    #[serde(default)]
    pub value: Option<String>,
    /// In-successful-tx flag.
    #[serde(default)]
    pub in_successful_tx: Option<bool>,
    /// Transaction hash.
    #[serde(default)]
    pub tx_hash: Option<String>,
}

/// Page of events plus cursor metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventsPage {
    /// Events.
    #[serde(default)]
    pub events: Vec<Event>,
    /// Latest ledger.
    #[serde(default)]
    pub latest_ledger: Option<u32>,
    /// Cursor for the next page.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Query builder for `getEvents`.
#[derive(Debug, Clone)]
pub struct EventsRequestBuilder<'a> {
    client: &'a RpcClient,
    start_ledger: Option<u32>,
    end_ledger: Option<u32>,
    filters: Vec<EventFilter>,
    cursor: Option<String>,
    limit: Option<u32>,
}

impl<'a> EventsRequestBuilder<'a> {
    /// Bind to an RPC client.
    pub fn new(client: &'a RpcClient) -> Self {
        Self {
            client,
            start_ledger: None,
            end_ledger: None,
            filters: Vec::new(),
            cursor: None,
            limit: None,
        }
    }

    /// Inclusive start ledger.
    pub fn start_ledger(mut self, ledger: u32) -> Self {
        self.start_ledger = Some(ledger);
        self
    }

    /// Inclusive end ledger.
    pub fn end_ledger(mut self, ledger: u32) -> Self {
        self.end_ledger = Some(ledger);
        self
    }

    /// Add a filter (topic / contract).
    pub fn filter(mut self, filter: EventFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Restrict to a single contract id.
    pub fn contract_id(self, id: impl Into<String>) -> Self {
        self.filter(EventFilter::contract(id))
    }

    /// Paging cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Max events to return.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute `getEvents`.
    pub async fn call(self) -> Result<EventsPage, SdkError> {
        #[derive(Serialize)]
        struct Pagination {
            #[serde(skip_serializing_if = "Option::is_none")]
            cursor: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
        }
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            start_ledger: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            end_ledger: Option<u32>,
            filters: Vec<EventFilter>,
            pagination: Pagination,
        }
        let params = Params {
            start_ledger: self.start_ledger,
            end_ledger: self.end_ledger,
            filters: self.filters,
            pagination: Pagination {
                cursor: self.cursor,
                limit: self.limit,
            },
        };
        self.client.call("getEvents", Some(params)).await
    }
}

impl RpcClient {
    /// Start a `getEvents` query builder.
    pub fn events(&self) -> EventsRequestBuilder<'_> {
        EventsRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_filter_defaults() {
        let f = EventFilter::contract("CABC").with_topic("AAAA");
        assert_eq!(f.event_type.as_deref(), Some("contract"));
        assert_eq!(f.contract_ids, ["CABC"]);
        assert_eq!(f.topics, vec![vec!["AAAA".to_string()]]);
    }
}

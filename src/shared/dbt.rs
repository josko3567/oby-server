use serde::{Deserialize, Serialize};

pub type VirtualTableID = String;
pub type OfferID        = String;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OrderID {
    pub table: VirtualTableID,
    pub count: u32
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VirtualTable {
    pub name: VirtualTableID,
    pub order_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OrderItem {
    pub id: OfferID,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Order {
    pub id: OrderID,
    pub finished: bool,
    pub items: Vec<OrderItem>
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Offer {
    pub name:           OfferID,
    pub description:    String,
    pub price_integer:  u32,
    pub price_fraction: u32,
}
/// Represents a product in the inventory.
#[derive(Debug, Clone)]
pub struct Product {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

impl Product {
    pub fn new(id: impl Into<String>, name: impl Into<String>, price: f64, quantity: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            price,
            quantity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub quantity: u32,
    pub total: f64,
    pub status: String,
}

impl Order {
    pub fn new(
        id: impl Into<String>,
        user_id: impl Into<String>,
        product_id: impl Into<String>,
        quantity: u32,
        total: f64,
    ) -> Self {
        Self {
            id: id.into(),
            user_id: user_id.into(),
            product_id: product_id.into(),
            quantity,
            total,
            status: "Created".to_string(),
        }
    }
}

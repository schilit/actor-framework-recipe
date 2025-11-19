// DTOs for Product
#[derive(Debug, Clone)]
pub struct ProductCreate {
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct ProductPatch {
    pub price: Option<f64>,
    pub quantity: Option<u32>,
}

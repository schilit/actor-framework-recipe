// Custom actions for Product
#[derive(Debug, Clone)]
pub enum ProductAction {
    CheckStock,
    ReserveStock(u32),
}

#[derive(Debug, Clone)]
pub enum ProductActionResult {
    StockLevel(u32),
    Reserved,
}

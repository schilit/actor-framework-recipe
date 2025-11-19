// Custom actions for Product
#[derive(Debug, Clone)]
pub enum ProductAction {
    #[allow(dead_code)]
    CheckStock,
    ReserveStock(u32),
}

#[derive(Debug, Clone)]
pub enum ProductActionResult {
    StockLevel(#[allow(dead_code)] u32),
    Reserved,
}

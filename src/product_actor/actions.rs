/// Custom actions for Product entities.
///
/// These actions represent domain-specific operations that can be performed
/// on a product beyond standard CRUD operations.
#[derive(Debug, Clone)]
pub enum ProductAction {
    /// Checks the current stock level without modifying it.
    #[allow(dead_code)]
    CheckStock,
    /// Reserves a specified amount of stock.
    ///
    /// # Arguments
    /// * `u32` - The quantity to reserve
    ///
    /// # Errors
    /// Will fail if the requested amount exceeds available stock.
    ReserveStock(u32),
}

/// Results from ProductActions - variants match 1:1 with ProductAction
#[derive(Debug, Clone)]
pub enum ProductActionResult {
    /// Result from CheckStock action - returns the current stock level
    CheckStock(u32),
    /// Result from ReserveStock action - returns unit on success
    ReserveStock(()),
}

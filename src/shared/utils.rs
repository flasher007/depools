//! Utility functions and helpers

/// Format amount with proper decimals
pub fn format_amount(amount: u64, decimals: u8) -> String {
    let value = amount as f64 / 10_f64.powi(decimals as i32);
    format!("{:.6}", value)
}

/// Calculate percentage change
pub fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
    if old_value > 0.0 {
        ((new_value - old_value) / old_value) * 100.0
    } else {
        0.0
    }
}

/// Generate unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

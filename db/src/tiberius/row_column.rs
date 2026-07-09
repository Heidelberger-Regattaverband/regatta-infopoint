use ::chrono::{DateTime, NaiveDate, Utc};
use ::tiberius::{Row, numeric::Decimal, time::chrono::NaiveDateTime};

/// Extension traits for `Row` to provide convenient methods for retrieving column values by name.
pub trait RowColumn<T: Default> {
    /// Retrieves the value of the specified column as type `T`. If the column is not found or cannot be converted to `T`, it will panic.
    fn get_column(&self, col_name: &str) -> T;
}

/// Extension trait for `Row` to provide a method for attempting to retrieve column values by name, returning `Option<T>`.
pub trait TryRowColumn<T: Default> {
    /// Attempts to retrieve the value of the specified column as type `T`. Returns `None` if the column is not found or cannot be converted to `T`.
    fn try_get_column(&self, col_name: &str) -> Option<T>;
}

/// Macro to implement `RowColumn` for multiple types, reducing boilerplate code.
macro_rules! impl_row_column {
    ($($type:ty),*) => { $(
        impl RowColumn<$type> for Row {
            fn get_column(&self, col_name: &str) -> $type {
                self.try_get::<$type, _>(col_name).unwrap().unwrap()
            }
        }
    )* };
}

/// Macro to implement `TryRowColumn` for multiple types, reducing boilerplate code.
macro_rules! impl_try_row_column {
    ($($type:ty),*) => { $(
        impl TryRowColumn<$type> for Row {
            fn try_get_column(&self, col_name: &str) -> Option<$type> {
                self.try_get::<$type, _>(col_name).unwrap_or_default()
            }
        }
    )* };
}

impl_row_column!(bool, u8, i16, i32, f32, f64, NaiveDateTime, NaiveDate);

impl_try_row_column!(bool, u8, i16, i32, f32, f64, Decimal, NaiveDateTime, NaiveDate);

impl RowColumn<String> for Row {
    fn get_column(&self, col_name: &str) -> String {
        self.try_get::<&str, _>(col_name).unwrap().unwrap().to_string()
    }
}

impl RowColumn<DateTime<Utc>> for Row {
    fn get_column(&self, col_name: &str) -> DateTime<Utc> {
        match self.try_get::<NaiveDateTime, _>(col_name) {
            Ok(value) => value
                .map(|date_time| DateTime::from_naive_utc_and_offset(date_time, Utc))
                .unwrap(),
            _ => DateTime::from_timestamp(0, 0).unwrap(),
        }
    }
}

impl TryRowColumn<String> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<String> {
        match self.try_get::<&str, _>(col_name) {
            Ok(Some(value)) => {
                if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            _ => None,
        }
    }
}

impl TryRowColumn<DateTime<Utc>> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<DateTime<Utc>> {
        match self.try_get::<NaiveDateTime, _>(col_name) {
            Ok(value) => value.map(|date_time| DateTime::from_naive_utc_and_offset(date_time, Utc)),
            _ => None,
        }
    }
}

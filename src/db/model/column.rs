use tiberius::{time::chrono::NaiveDateTime, Row};

// see: https://github.com/prisma/tiberius/issues/101#issuecomment-978144867
pub trait Column {
    fn get(row: &Row, col_name: &str) -> Self;
}

impl Column for i32 {
    fn get(row: &Row, col_name: &str) -> i32 {
        row.try_get::<i32, _>(col_name)
            .unwrap_or_default()
            .unwrap_or_default()
    }
}

impl Column for f32 {
    fn get(row: &Row, col_name: &str) -> f32 {
        row.try_get::<f32, _>(col_name).unwrap().unwrap_or_default()
    }
}

impl Column for f64 {
    fn get(row: &Row, col_name: &str) -> f64 {
        row.try_get::<f64, _>(col_name).unwrap().unwrap_or_default()
    }
}

impl Column for NaiveDateTime {
    fn get(row: &Row, col_name: &str) -> NaiveDateTime {
        row.try_get::<NaiveDateTime, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl Column for String {
    fn get(row: &Row, col_name: &str) -> String {
        row.try_get::<&str, _>(col_name)
            .unwrap()
            .unwrap_or_default()
            .to_string()
    }
}

impl Column for Option<String> {
    fn get(row: &Row, col_name: &str) -> Option<String> {
        match row.try_get::<&str, _>(col_name) {
            Ok(Some(value)) => Some(value.to_string()),
            _ => None,
        }
    }
}
impl Column for Option<i32> {
    fn get(row: &Row, col_name: &str) -> Option<i32> {
        match row.try_get::<i32, _>(col_name) {
            Ok(value) => value,
            _ => None,
        }
    }
}

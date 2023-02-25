use tiberius::Row;

// see: https://github.com/prisma/tiberius/issues/101#issuecomment-978144867
pub trait Column {
    fn get(row: &Row, col_name: &str) -> Self;
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

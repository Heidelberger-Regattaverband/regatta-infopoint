use tiberius::{time::chrono::NaiveDateTime, Row};

pub trait RowColumn<T> {
    fn get_column(&self, col_name: &str) -> T;
}

pub trait TryRowColumn<T> {
    fn try_get_column(&self, col_name: &str) -> Option<T>;
}

impl RowColumn<bool> for Row {
    fn get_column(&self, col_name: &str) -> bool {
        self.try_get::<bool, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<u8> for Row {
    fn get_column(&self, col_name: &str) -> u8 {
        self.try_get::<u8, _>(col_name).unwrap().unwrap_or_default()
    }
}

impl RowColumn<i16> for Row {
    fn get_column(&self, col_name: &str) -> i16 {
        self.try_get::<i16, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<i32> for Row {
    fn get_column(&self, col_name: &str) -> i32 {
        self.try_get::<i32, _>(col_name)
            .unwrap_or_default()
            .unwrap_or_default()
    }
}

impl RowColumn<f64> for Row {
    fn get_column(&self, col_name: &str) -> f64 {
        self.try_get::<f64, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<NaiveDateTime> for Row {
    fn get_column(&self, col_name: &str) -> NaiveDateTime {
        self.try_get::<NaiveDateTime, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<String> for Row {
    fn get_column(&self, col_name: &str) -> String {
        self.try_get::<&str, _>(col_name)
            .unwrap()
            .unwrap_or_default()
            .to_string()
    }
}

impl TryRowColumn<String> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<String> {
        match self.try_get::<&str, _>(col_name) {
            Ok(Some(value)) => Some(value.to_string()),
            _ => None,
        }
    }
}

impl TryRowColumn<i32> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<i32> {
        match self.try_get::<i32, _>(col_name) {
            Ok(value) => value,
            _ => None,
        }
    }
}

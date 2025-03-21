use chrono::{DateTime, NaiveDate, Utc};
use tiberius::{Row, numeric::Decimal, time::chrono::NaiveDateTime};

pub trait RowColumn<T>
where
    T: Default,
{
    fn get_column(&self, col_name: &str) -> T;
}

pub trait TryRowColumn<T>
where
    T: Default,
{
    fn try_get_column(&self, col_name: &str) -> Option<T>;
}

impl RowColumn<bool> for Row {
    fn get_column(&self, col_name: &str) -> bool {
        self.try_get::<bool, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<u8> for Row {
    fn get_column(&self, col_name: &str) -> u8 {
        self.try_get::<u8, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<i16> for Row {
    fn get_column(&self, col_name: &str) -> i16 {
        self.try_get::<i16, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<i32> for Row {
    fn get_column(&self, col_name: &str) -> i32 {
        self.try_get::<i32, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<f32> for Row {
    fn get_column(&self, col_name: &str) -> f32 {
        self.try_get::<f32, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<f64> for Row {
    fn get_column(&self, col_name: &str) -> f64 {
        self.try_get::<f64, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<NaiveDateTime> for Row {
    fn get_column(&self, col_name: &str) -> NaiveDateTime {
        self.try_get::<NaiveDateTime, _>(col_name).unwrap().unwrap()
    }
}

impl RowColumn<String> for Row {
    fn get_column(&self, col_name: &str) -> String {
        self.try_get::<&str, _>(col_name).unwrap().unwrap().to_string()
    }
}

impl RowColumn<NaiveDate> for Row {
    fn get_column(&self, col_name: &str) -> NaiveDate {
        self.try_get::<NaiveDate, _>(col_name).unwrap().unwrap()
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

impl TryRowColumn<i32> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<i32> {
        self.try_get::<i32, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<i16> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<i16> {
        self.try_get::<i16, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<u8> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<u8> {
        self.try_get::<u8, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<bool> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<bool> {
        self.try_get::<bool, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<NaiveDateTime> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<NaiveDateTime> {
        self.try_get::<NaiveDateTime, _>(col_name).unwrap_or_default()
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

impl TryRowColumn<f64> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<f64> {
        self.try_get::<f64, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<f32> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<f32> {
        self.try_get::<f32, _>(col_name).unwrap_or_default()
    }
}

impl TryRowColumn<Decimal> for Row {
    fn try_get_column(&self, col_name: &str) -> Option<Decimal> {
        self.try_get::<Decimal, _>(col_name).unwrap_or_default()
    }
}

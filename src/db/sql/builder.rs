use super::error::SqlBuilderError;
use anyhow::{Ok, Result};

enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone)]
pub struct SqlBuilder {
    /// The table to select from.
    table: String,
    /// The columns to select.
    columns: Option<Vec<String>>,
    /// The number of rows to return.
    limit: Option<i32>,
    /// The orders to apply to the query.
    orders: Option<Vec<(String, bool)>>,
    /// The where clauses to apply to the query.
    wheres: Option<Vec<String>>,
    /// The error that occurred while building the SQL query.
    error: Option<SqlBuilderError>,
}

impl SqlBuilder {
    /// Create a new SqlBuilder.
    /// # Arguments
    ///   table: The table to select from.
    /// # Returns
    ///  A new SqlBuilder.
    pub fn select_from<S: ToString>(table: S) -> Self {
        SqlBuilder {
            table: table.to_string(),
            columns: None,
            limit: None,
            orders: None,
            wheres: None,
            error: None,
        }
    }

    /// Set the columns to select.
    /// # Arguments
    ///    columns: The columns to select.
    /// # Returns
    ///   A mutable reference to the SqlBuilder.
    pub fn columns<S: ToString>(&mut self, columns: &[S]) -> &mut Self {
        self.columns = Some(columns.iter().map(|column| column.to_string()).collect());
        self
    }

    /// Set the number of rows to return.
    /// # Arguments
    ///     limit: The number of rows to return.
    /// # Returns
    ///    A mutable reference to the SqlBuilder.
    pub fn limit(&mut self, limit: i32) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Set the orders to apply to the query.
    /// # Arguments
    ///   orders: The orders to apply to the query.
    /// # Returns
    ///  A mutable reference to the SqlBuilder.
    pub fn order_by<S: ToString>(&mut self, orders: &[(S, bool)]) -> &mut Self {
        self.orders = Some(orders.iter().map(|(column, asc)| (column.to_string(), *asc)).collect());
        self
    }

    /// Set the where clauses to apply to the query.
    pub fn where_eq<S: ToString>(&mut self, column: S, value: S) -> &mut Self {
        if column.to_string().is_empty() {
            self.error = Some(SqlBuilderError::NoWhereColumn);
        }
        if value.to_string().is_empty() {
            self.error = Some(SqlBuilderError::NoWhereValue(column.to_string()));
        }

        let where_clause = format!("{} = {}", column.to_string(), value.to_string());
        self.wheres = match &self.wheres {
            Some(wheres) => Some(wheres.iter().map(|where_clause| where_clause.to_string()).collect()),
            None => Some(vec![where_clause]),
        };
        self
    }

    /// Build the SQL query.
    /// # Returns
    ///  The SQL query.
    /// # Errors
    /// SqlBuilderError::NoTableName: If the table name is not set.
    /// SqlBuilderError::NoColumnNames: If the column names are not set.
    pub fn build(&self) -> Result<String> {
        if let Some(error) = &self.error {
            return Err(error.clone().into());
        }
        if self.table.is_empty() {
            return Err(SqlBuilderError::NoTableName.into());
        }
        if self.columns.is_none() {
            return Err(SqlBuilderError::NoColumnNames.into());
        }

        let top = if self.limit.is_some() {
            format!(" TOP {}", self.limit.unwrap())
        } else {
            String::new()
        };

        let order_by = if self.orders.is_some() {
            let orders = self.orders.as_ref().unwrap();
            format!(
                " ORDER BY {}",
                orders
                    .iter()
                    .map(|(column, asc)| format!("{} {}", column, if *asc { "ASC" } else { "DESC" }))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else {
            String::new()
        };

        let wheres = if self.wheres.is_some() {
            let wheres = self.wheres.as_ref().unwrap();
            format!(" WHERE {}", wheres.join(" AND "))
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT{top} {} FROM {}{wheres}{order_by}",
            self.columns.as_ref().unwrap().join(", "),
            self.table
        );
        Ok(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_from() -> Result<()> {
        let select = SqlBuilder::select_from("Event")
            .limit(1)
            .columns(&["Event_ID", "Event_StartDate"])
            .order_by(&[("Event_StartDate", false), ("Event_ID", false)])
            .build()?;
        assert_eq!(
            select,
            "SELECT TOP 1 Event_ID, Event_StartDate FROM Event ORDER BY Event_StartDate DESC, Event_ID DESC"
        );
        Ok(())
    }

    #[test]
    fn test_select_from_wheres() -> Result<()> {
        let select = SqlBuilder::select_from("Event")
            .columns(&["*"])
            .where_eq("Event_ID", "14")
            .build()?;
        assert_eq!(select, "SELECT * FROM Event WHERE Event_ID = 14");
        Ok(())
    }

    #[test]
    fn test_select_from_no_where_column() -> Result<()> {
        let select = SqlBuilder::select_from("Event").where_eq("", "14").build();
        assert!(select.is_err());
        assert_eq!(select.err().unwrap().to_string(), "WHERE column not defined");
        Ok(())
    }

    #[test]
    fn test_select_from_no_column_names() -> Result<()> {
        let select = SqlBuilder::select_from("Event").build();
        assert!(select.is_err());
        Ok(())
    }

    #[test]
    fn test_select_from_no_table_name() -> Result<()> {
        let select = SqlBuilder::select_from("").build();
        assert!(select.is_err());
        Ok(())
    }
}

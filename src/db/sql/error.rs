use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SqlBuilderError {
    #[error("No table name")]
    NoTableName,
    #[error("No column names")]
    NoColumnNames,
    #[error("WHERE condition is empty")]
    NoWhereCond,
    #[error("WHERE column not defined")]
    NoWhereColumn,
    #[error("WHERE value for field \"{0}\" not defined")]
    NoWhereValue(String),
    #[error("WHERE list for field \"{0}\" not defined")]
    NoWhereList(String),
    #[error("WHERE query for field \"{0}\" not defined")]
    NoWhereQuery(String),
}

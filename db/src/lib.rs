//! Database layer for regatta management system.
//!
//! This crate provides database connectivity and data models for the regatta
//! information portal. It supports both Aquarius (regatta management) and
//! timekeeper data sources.

pub mod aquarius;
pub mod cache;
pub mod error;
pub mod tiberius;
pub mod timekeeper;

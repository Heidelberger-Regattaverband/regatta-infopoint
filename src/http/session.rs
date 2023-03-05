use actix_session::storage::{CookieSessionStore, LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use anyhow::Error;
use core::{future::Future, pin::Pin};
use log::info;
use std::collections::HashMap;

pub(crate) type SessionState = HashMap<String, String>;

pub struct InfoportalStore {
    cookie_store: CookieSessionStore,
}

impl InfoportalStore {
    pub fn new() -> Self {
        InfoportalStore {
            cookie_store: CookieSessionStore::default(),
        }
    }
}

impl SessionStore for InfoportalStore {
    fn load<'life0, 'life1, 'async_trait>(
        &'life0 self,
        session_key: &'life1 SessionKey,
    ) -> Pin<Box<dyn Future<Output = Result<Option<SessionState>, LoadError>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        info!("Load session state: key={:?}", session_key);
        self.cookie_store.load(session_key)
    }

    fn save<'life0, 'life1, 'async_trait>(
        &'life0 self,
        session_state: SessionState,
        ttl: &'life1 Duration,
    ) -> Pin<Box<dyn Future<Output = Result<SessionKey, SaveError>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        info!("Save session state.");
        self.cookie_store.save(session_state, ttl)
    }

    fn update<'life0, 'life1, 'async_trait>(
        &'life0 self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &'life1 Duration,
    ) -> Pin<Box<dyn Future<Output = Result<SessionKey, UpdateError>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        info!("update session: key={:?}, TTL={}", session_key, ttl);
        self.cookie_store.update(session_key, session_state, ttl)
    }

    fn update_ttl<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        session_key: &'life1 SessionKey,
        ttl: &'life2 Duration,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        info!("update session TTL: key={:?}, TTL={}", session_key, ttl);
        self.cookie_store.update_ttl(session_key, ttl)
    }

    fn delete<'life0, 'life1, 'async_trait>(
        &'life0 self,
        session_key: &'life1 SessionKey,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        info!("Delete session state: key={:?}", session_key);
        self.cookie_store.delete(session_key)
    }
}

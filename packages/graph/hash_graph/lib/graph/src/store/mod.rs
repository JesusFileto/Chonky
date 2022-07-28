pub mod crud;
pub mod error;

mod pool;
mod postgres;

use std::fmt;

use async_trait::async_trait;
use error_stack::{Context, Result};

use self::error::LinkActivationError;
pub use self::{
    error::{BaseUriAlreadyExists, BaseUriDoesNotExist, InsertionError, QueryError, UpdateError},
    pool::StorePool,
    postgres::{AsClient, PostgresStore, PostgresStorePool},
};
use crate::{
    knowledge::{Entity, EntityId, Link, Links},
    ontology::{
        types::{uri::VersionedUri, EntityType, LinkType, PropertyType},
        AccountId,
    },
    store::crud::Read,
    DataType,
};

#[derive(Debug)]
pub struct StoreError;

impl Context for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("The store encountered an error")
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ArgEnum))]
pub enum DatabaseType {
    #[default]
    Postgres,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct DatabaseConnectionInfo {
    /// The database type to connect to
    #[cfg_attr(feature = "clap", clap(long, default_value = "postgres", arg_enum))]
    database_type: DatabaseType,

    /// Database username
    #[cfg_attr(
        feature = "clap",
        clap(long, default_value = "postgres", env = "HASH_GRAPH_USER")
    )]
    user: String,

    /// Database password for authentication
    #[cfg_attr(
        feature = "clap",
        clap(long, default_value = "postgres", env = "HASH_GRAPH_PASSWORD")
    )]
    password: String,

    /// The host to connect to
    #[cfg_attr(
        feature = "clap",
        clap(long, default_value = "localhost", env = "HASH_GRAPH_HOST")
    )]
    host: String,

    /// The port to connect to
    #[cfg_attr(
        feature = "clap",
        clap(long, default_value = "5432", env = "HASH_GRAPH_PORT")
    )]
    port: u16,

    /// The database name to use
    #[cfg_attr(
        feature = "clap",
        clap(long, default_value = "graph", env = "HASH_GRAPH_DATABASE")
    )]
    database: String,
}

impl DatabaseConnectionInfo {
    #[must_use]
    pub const fn new(
        database_type: DatabaseType,
        user: String,
        password: String,
        host: String,
        port: u16,
        database: String,
    ) -> Self {
        Self {
            database_type,
            user,
            password,
            host,
            port,
            database,
        }
    }

    /// Creates a database connection url.
    ///
    /// Note, that this will reveal the password, so the returned output should not be printed. The
    /// [`Display`] implementation should be used instead, which will mask the password.
    ///
    /// [`Display`]: core::fmt::Display.
    #[must_use]
    pub fn url(&self) -> String {
        let db_type = match self.database_type {
            DatabaseType::Postgres => "postgres",
        };
        format!(
            "{}://{}:{}@{}:{}/{}",
            db_type, self.user, self.password, self.host, self.port, self.database
        )
    }

    #[must_use]
    pub const fn database_type(&self) -> DatabaseType {
        self.database_type
    }

    #[must_use]
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Returns the password in plain text.
    ///
    /// Note, that this will reveal the password, so the returned output should not be printed.
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    pub fn database(&self) -> &str {
        &self.database
    }
}

impl fmt::Display for DatabaseConnectionInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let db_type = match self.database_type {
            DatabaseType::Postgres => "postgres",
        };
        write!(
            fmt,
            "{}://{}:***@{}:{}/{}",
            db_type, self.user, self.host, self.port, self.database
        )
    }
}

/// Describes the API of a store implementation.
///
/// # Errors
///
/// In addition to the errors described in the methods of this trait, further errors might also be
/// raised depending on the implementation, e.g. connection issues.
pub trait Store =
    DataTypeStore + PropertyTypeStore + LinkTypeStore + EntityTypeStore + EntityStore + LinkStore;

/// Describes the API of a store implementation for [`DataType`]s.
#[async_trait]
pub trait DataTypeStore {
    /// Creates a new [`DataType`].
    ///
    /// # Errors:
    ///
    /// - if the account referred to by `created_by` does not exist.
    /// - if the [`BaseUri`] of the `data_type` already exist.
    ///
    /// [`BaseUri`]: crate::ontology::types::uri::BaseUri
    async fn create_data_type(
        &mut self,
        data_type: &DataType,
        created_by: AccountId,
    ) -> Result<(), InsertionError>;

    /// Get the [`DataType`] specified by `identifier`.
    ///
    /// # Errors
    ///
    /// - if the requested [`DataType`] doesn't exist.
    async fn get_data_type<'i, I: Send>(&self, identifier: I) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, DataType>,
    {
        self.get(identifier).await
    }

    /// Update the definition of an existing [`DataType`].
    ///
    /// # Errors
    ///
    /// - if the [`DataType`] doesn't exist.
    async fn update_data_type(
        &mut self,
        data_type: &DataType,
        updated_by: AccountId,
    ) -> Result<(), UpdateError>;
}

/// Describes the API of a store implementation for [`PropertyType`]s.
#[async_trait]
pub trait PropertyTypeStore {
    /// Creates a new [`PropertyType`].
    ///
    /// # Errors:
    ///
    /// - if the account referred to by `created_by` does not exist.
    /// - if the [`BaseUri`] of the `property_type` already exists.
    ///
    /// [`BaseUri`]: crate::ontology::types::uri::BaseUri
    async fn create_property_type(
        &mut self,
        property_type: &PropertyType,
        created_by: AccountId,
    ) -> Result<(), InsertionError>;

    /// Get the [`PropertyType`] specified by `identifier`.
    ///
    /// # Errors
    ///
    /// - if the requested [`PropertyType`] doesn't exist.
    async fn get_property_type<'i, I: Send>(
        &self,
        identifier: I,
    ) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, PropertyType>,
    {
        self.get(identifier).await
    }

    /// Update the definition of an existing [`PropertyType`].
    ///
    /// # Errors
    ///
    /// - if the [`PropertyType`] doesn't exist.
    async fn update_property_type(
        &mut self,
        property_type: &PropertyType,
        updated_by: AccountId,
    ) -> Result<(), UpdateError>;
}

/// Describes the API of a store implementation for [`EntityType`]s.
#[async_trait]
pub trait EntityTypeStore {
    /// Creates a new [`EntityType`].
    ///
    /// # Errors:
    ///
    /// - if the account referred to by `created_by` does not exist.
    /// - if the [`BaseUri`] of the `entity_type` already exist.
    ///
    /// [`BaseUri`]: crate::ontology::types::uri::BaseUri
    async fn create_entity_type(
        &mut self,
        entity_type: &EntityType,
        created_by: AccountId,
    ) -> Result<(), InsertionError>;

    /// Get the [`EntityType`] specified by `identifier`.
    ///
    /// # Errors
    ///
    /// - if the requested [`EntityType`] doesn't exist.
    async fn get_entity_type<'i, I: Send>(&self, identifier: I) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, EntityType>,
    {
        self.get(identifier).await
    }

    /// Update the definition of an existing [`EntityType`].
    ///
    /// # Errors
    ///
    /// - if the [`EntityType`] doesn't exist.
    async fn update_entity_type(
        &mut self,
        entity_type: &EntityType,
        updated_by: AccountId,
    ) -> Result<(), UpdateError>;
}

/// Describes the API of a store implementation for [`LinkType`]s.
#[async_trait]
pub trait LinkTypeStore {
    /// Creates a new [`LinkType`].
    ///
    /// # Errors:
    ///
    /// - if the account referred to by `created_by` does not exist.
    /// - if the [`BaseUri`] of the `property_type` already exists.
    ///
    /// [`BaseUri`]: crate::ontology::types::uri::BaseUri
    async fn create_link_type(
        &mut self,
        link_type: &LinkType,
        created_by: AccountId,
    ) -> Result<(), InsertionError>;

    /// Get the [`LinkType`] specified by `identifier`.
    ///
    /// # Errors
    ///
    /// - if the requested [`LinkType`] doesn't exist.
    async fn get_link_type<'i, I: Send>(&self, identifier: I) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, LinkType>,
    {
        self.get(identifier).await
    }

    /// Update the definition of an existing [`LinkType`].
    ///
    /// # Errors
    ///
    /// - if the [`LinkType`] doesn't exist.
    async fn update_link_type(
        &mut self,
        property_type: &LinkType,
        updated_by: AccountId,
    ) -> Result<(), UpdateError>;
}

/// Describes the API of a store implementation for Entities.
#[async_trait]
pub trait EntityStore {
    /// Creates a new [`Entity`].
    ///
    /// # Errors:
    ///
    /// - if the [`EntityType`] doesn't exist
    /// - if the [`Entity`] is not valid with respect to the specified [`EntityType`]
    /// - if the account referred to by `created_by` does not exist
    async fn create_entity(
        &mut self,
        entity: &Entity,
        entity_type_uri: VersionedUri,
        created_by: AccountId,
    ) -> Result<EntityId, InsertionError>;

    /// Get the [`Entity`] specified by `identifier`.
    ///
    /// Depending on the `identifier` the output is specified by [`Read::Output`].
    ///
    /// # Errors
    ///
    /// - if the requested [`Entity`] doesn't exist
    async fn get_entity<'i, I: Send>(&self, identifier: I) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, Entity>,
    {
        self.get(identifier).await
    }

    /// Update an existing [`Entity`].
    ///
    /// # Errors
    ///
    /// - if the [`Entity`] doesn't exist
    /// - if the [`EntityType`] doesn't exist
    /// - if the [`Entity`] is not valid with respect to its [`EntityType`]
    /// - if the account referred to by `updated_by` does not exist
    async fn update_entity(
        &mut self,
        entity_id: EntityId,
        entity: &Entity,
        entity_type_uri: VersionedUri,
        updated_by: AccountId,
    ) -> Result<(), UpdateError>;
}

/// Describes the API of a store implementation for [`Link`]s.
#[async_trait]
pub trait LinkStore {
    /// Creates a new [`Link`].
    ///
    /// # Errors:
    ///
    /// - if the [`LinkType`] doesn't exist
    /// - if the [`Link`] already exists
    /// - if the account referred to by `created_by` does not exist
    async fn create_link(
        &mut self,
        link: &Link,
        created_by: AccountId,
    ) -> Result<(), InsertionError>;

    /// Get [`Links`] of an [`Entity`] identified by an [`EntityId`].
    ///
    /// # Errors
    ///
    /// - if the requested [`Entity`] doesn't exist
    async fn get_entity_links<'i, I: Send>(&self, identifier: I) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, I, Links>,
    {
        self.get(identifier).await
    }

    /// Get a [`Link`] target identified by an [`EntityId`] and a Link Type [`VersionedUri`].
    ///
    /// # Errors
    ///
    /// - if the requested [`Entity`] doesn't exist
    async fn get_link_target<'i, E: Send, L: Send>(
        &self,
        source_entity: E,
        link_type: L,
    ) -> Result<Self::Output, QueryError>
    where
        Self: Read<'i, (E, L), Links>,
    {
        self.get((source_entity, link_type)).await
    }

    /// Inactivates a [`Link`] between a source and target [`Entity`].
    ///
    /// # Errors:
    ///
    /// - if the [`Link`] doesn't exist
    /// - if the account referred to by `created_by` does not exist
    async fn inactivate_link(&mut self, link: &Link) -> Result<(), LinkActivationError>;
}

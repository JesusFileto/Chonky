use alloc::borrow::Cow;
use core::{fmt, str::FromStr};

use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};
use type_system::url::BaseUrl;
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

use crate::{
    entity_type::{EntityTypeQueryPath, EntityTypeQueryPathVisitor},
    filter::{JsonPath, ParameterType, PathToken, QueryPath, parse_query_token},
    subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind, SharedEdgeKind},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityQueryPath<'p> {
    /// The [`EntityUuid`] of the [`EntityId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::Uuid);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityUuid`]: graph_types::knowledge::entity::EntityUuid
    /// [`EntityId`]: graph_types::knowledge::entity::EntityId
    Uuid,
    /// The [`OwnedById`] of the [`EntityId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["ownedById"]))?;
    /// assert_eq!(path, EntityQueryPath::OwnedById);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`OwnedById`]: graph_types::owned_by_id::OwnedById
    /// [`EntityId`]: graph_types::knowledge::entity::EntityId
    OwnedById,
    /// The [`DraftId`] of the [`EntityId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["draftId"]))?;
    /// assert_eq!(path, EntityQueryPath::DraftId);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`DraftId`]: graph_types::knowledge::entity::DraftId
    /// [`EntityId`]: graph_types::knowledge::entity::EntityId
    DraftId,
    /// The [`EntityEditionId`] of the [`EntityRecordId`] belonging to the [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["editionId"]))?;
    /// assert_eq!(path, EntityQueryPath::EditionId);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityEditionId`]: graph_types::knowledge::entity::EntityEditionId
    /// [`EntityRecordId`]: graph_types::knowledge::entity::EntityRecordId
    EditionId,
    /// The decision time axis of the [`EntityTemporalMetadata`] belonging to the [`Entity`].
    ///
    /// It's not possible to query for the temporal axis directly, this has to be done via the
    /// `temporalAxes` parameter on the request. The decision time is returned as part of
    /// [`EntityTemporalMetadata`] of the [`EntityMetadata`].
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityMetadata`]: graph_types::knowledge::entity::EntityMetadata
    /// [`EntityTemporalMetadata`]: graph_types::knowledge::entity::EntityTemporalMetadata
    DecisionTime,
    /// The transaction time axis of the [`EntityTemporalMetadata`] belonging to the [`Entity`].
    ///
    /// It's not possible to query for the temporal axis directly, this has to be done via the
    /// `temporalAxes` parameter on the request. The transaction time is returned as part
    /// of [`EntityTemporalMetadata`] of the [`EntityMetadata`].
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityMetadata`]: graph_types::knowledge::entity::EntityMetadata
    /// [`EntityTemporalMetadata`]: graph_types::knowledge::entity::EntityTemporalMetadata
    TransactionTime,
    /// The list of [`EntityType`]s' [`BaseUrl`]s belonging to the [`Entity`].
    ///
    /// It's currently not possible to query for the list of types directly. Use [`EntityTypeEdge`]
    /// instead.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`BaseUrl`]: type_system::url::BaseUrl
    /// [`EntityType`]: type_system::schema::EntityType
    /// [`EntityTypeEdge`]: Self::EntityTypeEdge
    TypeBaseUrls,
    /// The list of [`EntityType`]s' versions belonging to the [`Entity`].
    ///
    /// It's currently not possible to query for the list of types directly. Use [`EntityTypeEdge`]
    /// instead.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityType`]: type_system::schema::EntityType
    /// [`EntityTypeEdge`]: Self::EntityTypeEdge
    TypeVersions,
    /// The confidence value for the [`Entity`].
    ///
    /// It's currently not possible to query for the entity confidence value directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    EntityConfidence,
    /// The confidence value for the [`Entity`]'s left entity link.
    ///
    /// It's currently not possible to query for the entity confidence value directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    LeftEntityConfidence,
    /// The provenance for the [`Entity`]'s left entity link.
    ///
    /// It's currently not possible to query for the link provenance value directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    LeftEntityProvenance,
    /// The confidence value for the [`Entity`]'s right entity link.
    ///
    /// It's currently not possible to query for the entity confidence value directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    RightEntityConfidence,
    /// The list of all property pointers of an [`Entity`].
    ///
    /// It's currently not possible to query for the list of property pointers directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    RightEntityProvenance,
    /// Whether or not the [`Entity`] is in a draft state.
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["archived"]))?;
    /// assert_eq!(path, EntityQueryPath::Archived);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    Archived,
    /// An edge from this [`Entity`] to it's [`EntityType`] using a [`SharedEdgeKind`].
    ///
    /// The corresponding reversed edge is [`EntityTypeQueryPath::EntityEdge`].
    ///
    /// Deserializes from `["type", ...]` where `...` is a path to a field of an [`EntityType`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::{entity::EntityQueryPath, entity_type::EntityTypeQueryPath};
    /// # use hash_graph_store::subgraph::edges::SharedEdgeKind;
    /// let path = EntityQueryPath::deserialize(json!(["type", "baseUrl"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityTypeEdge {
    ///     edge_kind: SharedEdgeKind::IsOfType,
    ///     path: EntityTypeQueryPath::BaseUrl,
    ///     inheritance_depth: None,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// It's possible to specify the inheritance search depths:
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::{entity::EntityQueryPath, entity_type::EntityTypeQueryPath};
    /// # use hash_graph_store::subgraph::edges::SharedEdgeKind;
    /// let path = EntityQueryPath::deserialize(json!(["type(inheritanceDepth = 10)", "baseUrl"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityTypeEdge {
    ///     edge_kind: SharedEdgeKind::IsOfType,
    ///     path: EntityTypeQueryPath::BaseUrl,
    ///     inheritance_depth: Some(10),
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`EntityType`]: type_system::schema::PropertyType
    EntityTypeEdge {
        edge_kind: SharedEdgeKind,
        path: EntityTypeQueryPath<'p>,
        inheritance_depth: Option<u32>,
    },
    /// An edge between two [`Entities`][`Entity`] using a [`KnowledgeGraphEdgeKind`].
    ///
    ///
    /// # Left entity
    ///
    /// Corresponds to the [`Entity`] specified by [`LinkData::left_entity_id()`].
    ///
    /// Deserializes from `["leftEntity", ...]` where `...` is the path of the left [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// # use hash_graph_store::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["leftEntity", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Outgoing,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`LinkData::left_entity_id()`]: graph_types::knowledge::link::LinkData::left_entity_id
    ///
    ///
    /// # Right entity
    ///
    /// Corresponds to the [`Entity`] specified by [`LinkData::right_entity_id()`].
    ///
    /// Deserializes from `["rightEntity", ...]` where `...` is the path of the left [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// # use hash_graph_store::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["rightEntity", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Outgoing,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`LinkData::right_entity_id()`]: graph_types::knowledge::link::LinkData::right_entity_id
    ///
    ///
    /// # Incoming links
    ///
    /// Represents an [`Entity`] linked from this [`Entity`].
    ///
    /// Deserializes from `["incomingLinks", ...]` where `...` is the path of the target
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// # use hash_graph_store::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["incomingLinks", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Incoming,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    ///
    /// # Outgoing links
    ///
    /// Represents an [`Entity`] linked from this [`Entity`].
    ///
    /// Deserializes from `["outgoingLinks", ...]` where `...` is the path of the target
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// # use hash_graph_store::subgraph::edges::{EdgeDirection, KnowledgeGraphEdgeKind};
    /// let path = EntityQueryPath::deserialize(json!(["outgoingLinks", "uuid"]))?;
    /// assert_eq!(path, EntityQueryPath::EntityEdge {
    ///     edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
    ///     path: Box::new(EntityQueryPath::Uuid),
    ///     direction: EdgeDirection::Incoming,
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    EntityEdge {
        edge_kind: KnowledgeGraphEdgeKind,
        path: Box<Self>,
        direction: EdgeDirection,
    },
    /// Corresponds to [`Entity::properties`].
    ///
    /// Deserializes from `["properties", ...]` where `...` is a path to a property URL of an
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!([
    ///     "properties",
    ///     "https://blockprotocol.org/@blockprotocol/types/property-type/address/"
    /// ]))?;
    /// assert_eq!(
    ///     path.to_string(),
    ///     r#"properties.$."https://blockprotocol.org/@blockprotocol/types/property-type/address/""#
    /// );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// It is possible to also refer to the value's data type ID or the canonical value:
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!([
    ///     "properties",
    ///     "https://blockprotocol.org/@blockprotocol/types/property-type/length/",
    ///     "dataTypeId"
    /// ]))?;
    /// # assert_eq!(
    /// #     path.to_string(),
    /// #     r#"propertyMetadata.$."value"."https://blockprotocol.org/@blockprotocol/types/property-type/length/"."metadata"."dataTypeId""#
    /// # );
    ///
    /// let path = EntityQueryPath::deserialize(json!([
    ///     "properties",
    ///     "https://blockprotocol.org/@blockprotocol/types/property-type/length/",
    ///     "convert",
    ///     "http://localhost:3000/@alice/types/data-type/meter/"
    /// ]))?;
    /// # assert_eq!(
    /// #     path.to_string(),
    /// #     r#"propertyMetadata.$."value"."https://blockprotocol.org/@blockprotocol/types/property-type/length/"."metadata"."canonical"."http://localhost:3000/@alice/types/data-type/meter/""#
    /// # );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    /// [`Entity::properties`]: graph_types::knowledge::entity::Entity::properties
    Properties(Option<JsonPath<'p>>),
    /// The property defined as [`label_property`] in the corresponding entity type metadata.
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["label"]))?;
    /// assert_eq!(path, EntityQueryPath::Label {
    ///     inheritance_depth: None
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// It's possible to specify the inheritance search depths:
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["label(inheritanceDepth = 10)"]))?;
    /// assert_eq!(path, EntityQueryPath::Label {
    ///     inheritance_depth: Some(10)
    /// });
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`label_property`]: type_system::schema::EntityType::label_property
    Label { inheritance_depth: Option<u32> },
    /// Corresponds to the provenance data of the [`Entity`].
    ///
    /// Deserializes from `["provenance", ...]` where `...` is a path to a provenance entry of an
    /// [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["provenance", "createdById"]))?;
    /// assert_eq!(path.to_string(), r#"provenance.$."createdById""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    Provenance(Option<JsonPath<'p>>),
    /// Corresponds to the provenance data of the [`Entity`].
    ///
    /// Deserializes from `["editionProvenance", ...]` where `...` is a path to a provenance entry
    /// of an [`Entity`].
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["editionProvenance", "createdById"]))?;
    /// assert_eq!(path.to_string(), r#"editionProvenance.$."createdById""#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    EditionProvenance(Option<JsonPath<'p>>),
    /// Corresponds to the metadata data of the properties of the [`Entity`].
    ///
    /// It's currently not possible to query for the list of property provenance values directly.
    ///
    /// [`Entity`]: graph_types::knowledge::entity::Entity
    PropertyMetadata(Option<JsonPath<'p>>),
    /// The embedding for the whole entity blob.
    ///
    /// Deserializes from `["embedding"]`:
    ///
    /// ```rust
    /// # use serde::Deserialize;
    /// # use serde_json::json;
    /// # use hash_graph_store::entity::EntityQueryPath;
    /// let path = EntityQueryPath::deserialize(json!(["embedding"]))?;
    /// assert_eq!(path, EntityQueryPath::Embedding);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    Embedding,
}

impl fmt::Display for EntityQueryPath<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid => fmt.write_str("uuid"),
            Self::OwnedById => fmt.write_str("ownedById"),
            Self::DraftId => fmt.write_str("draftId"),
            Self::EditionId => fmt.write_str("editionId"),
            Self::DecisionTime => fmt.write_str("decisionTime"),
            Self::TransactionTime => fmt.write_str("transactionTime"),
            Self::TypeBaseUrls => fmt.write_str("typeBaseUrls"),
            Self::TypeVersions => fmt.write_str("typeVersions"),
            Self::Archived => fmt.write_str("archived"),
            Self::Properties(Some(property)) => write!(fmt, "properties.{property}"),
            Self::Properties(None) => fmt.write_str("properties"),
            Self::Provenance(Some(path)) => write!(fmt, "provenance.{path}"),
            Self::Provenance(None) => fmt.write_str("provenance"),
            Self::Label { .. } => fmt.write_str("label"),
            Self::EditionProvenance(Some(path)) => write!(fmt, "editionProvenance.{path}"),
            Self::EditionProvenance(None) => fmt.write_str("editionProvenance"),
            Self::PropertyMetadata(Some(path)) => write!(fmt, "propertyMetadata.{path}"),
            Self::PropertyMetadata(None) => fmt.write_str("propertyMetadata"),
            Self::Embedding => fmt.write_str("embedding"),
            Self::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path,
                inheritance_depth: Some(depth),
            } => write!(fmt, "type({depth}).{path}"),
            Self::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path,
                inheritance_depth: None,
            } => write!(fmt, "type.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path,
                direction: EdgeDirection::Outgoing,
            } => write!(fmt, "leftEntity.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path,
                direction: EdgeDirection::Outgoing,
            } => write!(fmt, "rightEntity.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path,
                direction: EdgeDirection::Incoming,
            } => write!(fmt, "outgoingLinks.{path}"),
            Self::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path,
                direction: EdgeDirection::Incoming,
            } => write!(fmt, "incomingLinks.{path}"),
            Self::EntityConfidence => fmt.write_str("entityConfidence"),
            Self::LeftEntityConfidence => fmt.write_str("leftEntityConfidence"),
            Self::LeftEntityProvenance => fmt.write_str("leftEntityProvenance"),
            Self::RightEntityConfidence => fmt.write_str("rightEntityConfidence"),
            Self::RightEntityProvenance => fmt.write_str("rightEntityProvenance"),
        }
    }
}

impl QueryPath for EntityQueryPath<'_> {
    fn expected_type(&self) -> ParameterType {
        match self {
            Self::EditionId | Self::Uuid | Self::OwnedById | Self::DraftId => ParameterType::Uuid,
            Self::DecisionTime | Self::TransactionTime => ParameterType::TimeInterval,
            Self::TypeBaseUrls => ParameterType::Vector(Box::new(ParameterType::VersionedUrl)),
            Self::TypeVersions => {
                ParameterType::Vector(Box::new(ParameterType::OntologyTypeVersion))
            }
            Self::Properties(_)
            | Self::Label { .. }
            | Self::Provenance(_)
            | Self::EditionProvenance(_)
            | Self::PropertyMetadata(_)
            | Self::LeftEntityProvenance
            | Self::RightEntityProvenance => ParameterType::Any,
            Self::EntityConfidence | Self::LeftEntityConfidence | Self::RightEntityConfidence => {
                ParameterType::F64
            }
            Self::Embedding => ParameterType::Vector(Box::new(ParameterType::F64)),
            Self::Archived => ParameterType::Boolean,
            Self::EntityTypeEdge { path, .. } => path.expected_type(),
            Self::EntityEdge { path, .. } => path.expected_type(),
        }
    }
}

/// A single token in an [`EntityQueryPath`].
#[derive(Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub enum EntityQueryToken {
    // TODO: we want to expose `EntityId` here instead
    Uuid,
    EditionId,
    DraftId,
    Archived,
    OwnedById,
    Type,
    Properties,
    Label,
    Provenance,
    EditionProvenance,
    Embedding,
    IncomingLinks,
    OutgoingLinks,
    LeftEntity,
    RightEntity,
}

/// Deserializes an [`EntityQueryPath`] from a string sequence.
pub(crate) struct EntityQueryPathVisitor {
    /// The current position in the sequence when deserializing.
    position: usize,
}

impl EntityQueryPathVisitor {
    pub(crate) const EXPECTING: &'static str =
        "one of `uuid`, `editionId`, `draftId`, `archived`, `ownedById`, `type`, `properties`, \
         `label`, `provenance`, `editionProvenance`, `embedding`, `incomingLinks`, \
         `outgoingLinks`, `leftEntity`, `rightEntity`";

    #[must_use]
    pub(crate) const fn new(position: usize) -> Self {
        Self { position }
    }
}

struct EntityPropertiesPathVisitor {
    position: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum MetaTag {
    Convert,
    DataTypeId,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PropertiesToken<'k> {
    Property(Cow<'k, BaseUrl>),
    Index(usize),
    Meta(MetaTag),
}

impl<'de> Visitor<'de> for EntityPropertiesPathVisitor {
    type Value = EntityQueryPath<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of property path elements")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut path_tokens = Vec::new();
        let mut is_metadata_path = false;
        while let Some(token) = seq.next_element::<PropertiesToken<'de>>()? {
            match token {
                PropertiesToken::Property(base_url) => {
                    path_tokens.push(PathToken::Field(Cow::Owned(base_url.to_string())));
                }
                PropertiesToken::Index(index) => {
                    if is_metadata_path {
                        return Err(de::Error::custom("Unexpected index found in property path"));
                    }
                    path_tokens.push(PathToken::Index(index));
                }
                PropertiesToken::Meta(meta) => {
                    // We convert the underlying value so we look at the property metadata's
                    // canonical value instead of the actual value
                    if is_metadata_path {
                        return Err(de::Error::custom(
                            "Unexpected meta tag found in property path",
                        ));
                    }
                    path_tokens = path_tokens
                        .into_iter()
                        .flat_map(|token| [PathToken::Field(Cow::Borrowed("value")), token])
                        .chain([PathToken::Field(Cow::Borrowed("metadata"))])
                        .collect();
                    is_metadata_path = true;

                    match meta {
                        MetaTag::Convert => {
                            path_tokens.push(PathToken::Field(Cow::Borrowed("canonical")));
                        }
                        MetaTag::DataTypeId => {
                            path_tokens.push(PathToken::Field(Cow::Borrowed("dataTypeId")));
                        }
                    }
                }
            }
            self.position += 1;
        }

        let json_path = (!path_tokens.is_empty()).then(|| JsonPath::from_path_tokens(path_tokens));
        if is_metadata_path {
            Ok(EntityQueryPath::PropertyMetadata(json_path))
        } else {
            Ok(EntityQueryPath::Properties(json_path))
        }
    }
}

impl<'de> Visitor<'de> for EntityQueryPathVisitor {
    type Value = EntityQueryPath<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(Self::EXPECTING)
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let query_token: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(self.position, &self))?;
        let (token, mut parameters) = parse_query_token(&query_token)?;
        self.position += 1;

        let query_path = match token {
            EntityQueryToken::Uuid => EntityQueryPath::Uuid,
            EntityQueryToken::EditionId => EntityQueryPath::EditionId,
            EntityQueryToken::OwnedById => EntityQueryPath::OwnedById,
            EntityQueryToken::DraftId => EntityQueryPath::DraftId,
            EntityQueryToken::Archived => EntityQueryPath::Archived,
            EntityQueryToken::Embedding => EntityQueryPath::Embedding,
            EntityQueryToken::Type => EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPathVisitor::new(self.position).visit_seq(seq)?,
                inheritance_depth: parameters
                    .remove("inheritanceDepth")
                    .map(u32::from_str)
                    .transpose()
                    .map_err(de::Error::custom)?,
            },
            EntityQueryToken::Properties => EntityPropertiesPathVisitor {
                position: self.position,
            }
            .visit_seq(seq)?,
            EntityQueryToken::Label => EntityQueryPath::Label {
                inheritance_depth: parameters
                    .remove("inheritanceDepth")
                    .map(u32::from_str)
                    .transpose()
                    .map_err(de::Error::custom)?,
            },
            EntityQueryToken::Provenance => {
                let mut path_tokens = Vec::new();
                while let Some(property) = seq.next_element::<PathToken<'de>>()? {
                    path_tokens.push(property);
                    self.position += 1;
                }

                if path_tokens.is_empty() {
                    EntityQueryPath::Provenance(None)
                } else {
                    EntityQueryPath::Provenance(Some(JsonPath::from_path_tokens(path_tokens)))
                }
            }
            EntityQueryToken::EditionProvenance => {
                let mut path_tokens = Vec::new();
                while let Some(property) = seq.next_element::<PathToken<'de>>()? {
                    path_tokens.push(property);
                    self.position += 1;
                }

                if path_tokens.is_empty() {
                    EntityQueryPath::EditionProvenance(None)
                } else {
                    EntityQueryPath::EditionProvenance(Some(JsonPath::from_path_tokens(
                        path_tokens,
                    )))
                }
            }
            EntityQueryToken::LeftEntity => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Outgoing,
            },
            EntityQueryToken::RightEntity => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Outgoing,
            },
            EntityQueryToken::OutgoingLinks => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Incoming,
            },
            EntityQueryToken::IncomingLinks => EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasRightEntity,
                path: Box::new(Self::new(self.position).visit_seq(seq)?),
                direction: EdgeDirection::Incoming,
            },
        };

        if !parameters.is_empty() {
            return Err(de::Error::custom(format!(
                "unknown parameters: {}",
                parameters.into_keys().collect::<Vec<_>>().join(", ")
            )));
        }

        Ok(query_path)
    }
}

impl<'de: 'p, 'p> Deserialize<'de> for EntityQueryPath<'p> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(EntityQueryPathVisitor::new(0))
    }
}

#[derive(Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[serde(rename_all = "camelCase")]
pub enum EntityQuerySortingToken {
    Uuid,
    Archived,
    Properties,
    Label,
    RecordCreatedAtTransactionTime,
    RecordCreatedAtDecisionTime,
    CreatedAtTransactionTime,
    CreatedAtDecisionTime,
    TypeTitle,
}

/// Deserializes an [`EntityQueryPath`] from a string sequence.
pub(crate) struct EntityQuerySortingVisitor {
    /// The current position in the sequence when deserializing.
    position: usize,
}

impl EntityQuerySortingVisitor {
    pub(crate) const EXPECTING: &'static str =
        "one of `uuid`, `archived`, `properties`, `label`, `recordCreatedAtTransactionTime`, \
         `recordCreatedAtDecisionTime`, `createdAtTransactionTime`, `createdAtDecisionTime`, \
         `typeTitle`";

    #[must_use]
    pub(crate) const fn new(position: usize) -> Self {
        Self { position }
    }
}

impl<'de> Visitor<'de> for EntityQuerySortingVisitor {
    type Value = EntityQueryPath<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(Self::EXPECTING)
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let query_token: String = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(self.position, &self))?;
        let (token, mut parameters) = parse_query_token(&query_token)?;
        self.position += 1;
        Ok(match token {
            EntityQuerySortingToken::Uuid => EntityQueryPath::Uuid,
            EntityQuerySortingToken::Archived => EntityQueryPath::Archived,
            EntityQuerySortingToken::RecordCreatedAtTransactionTime => {
                EntityQueryPath::TransactionTime
            }
            EntityQuerySortingToken::RecordCreatedAtDecisionTime => EntityQueryPath::DecisionTime,
            EntityQuerySortingToken::CreatedAtTransactionTime => {
                EntityQueryPath::Provenance(Some(JsonPath::from_path_tokens(vec![
                    PathToken::Field(Cow::Borrowed("createdAtTransactionTime")),
                ])))
            }
            EntityQuerySortingToken::CreatedAtDecisionTime => {
                EntityQueryPath::Provenance(Some(JsonPath::from_path_tokens(vec![
                    PathToken::Field(Cow::Borrowed("createdAtDecisionTime")),
                ])))
            }
            EntityQuerySortingToken::TypeTitle => EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPath::Title,
                inheritance_depth: Some(0),
            },
            EntityQuerySortingToken::Label => EntityQueryPath::Label {
                inheritance_depth: parameters
                    .remove("inheritanceDepth")
                    .map(u32::from_str)
                    .transpose()
                    .map_err(de::Error::custom)?,
            },
            EntityQuerySortingToken::Properties => EntityPropertiesPathVisitor {
                position: self.position,
            }
            .visit_seq(seq)?,
        })
    }
}

impl<'de: 'p, 'p> EntityQueryPath<'p> {
    /// Deserializes an [`EntityQueryPath`] from a string sequence represeting sorting keys
    ///
    /// # Errors
    ///
    /// If the sequence could not be deserialized
    pub fn deserialize_from_sorting_tokens<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(EntityQuerySortingVisitor::new(0))
    }

    #[must_use]
    pub fn into_owned(self) -> EntityQueryPath<'static> {
        match self {
            Self::Uuid => EntityQueryPath::Uuid,
            Self::OwnedById => EntityQueryPath::OwnedById,
            Self::DraftId => EntityQueryPath::DraftId,
            Self::EditionId => EntityQueryPath::EditionId,
            Self::DecisionTime => EntityQueryPath::DecisionTime,
            Self::TransactionTime => EntityQueryPath::TransactionTime,
            Self::TypeBaseUrls => EntityQueryPath::TypeBaseUrls,
            Self::TypeVersions => EntityQueryPath::TypeVersions,
            Self::Archived => EntityQueryPath::Archived,
            Self::EntityTypeEdge {
                path,
                edge_kind,
                inheritance_depth,
            } => EntityQueryPath::EntityTypeEdge {
                path: path.into_owned(),
                edge_kind,
                inheritance_depth,
            },
            Self::EntityEdge {
                path,
                edge_kind,
                direction,
            } => EntityQueryPath::EntityEdge {
                path: Box::new(path.into_owned()),
                edge_kind,
                direction,
            },
            Self::Properties(path) => EntityQueryPath::Properties(path.map(JsonPath::into_owned)),
            Self::Label { inheritance_depth } => EntityQueryPath::Label { inheritance_depth },
            Self::Embedding => EntityQueryPath::Embedding,
            Self::EntityConfidence => EntityQueryPath::EntityConfidence,
            Self::LeftEntityConfidence => EntityQueryPath::LeftEntityConfidence,
            Self::LeftEntityProvenance => EntityQueryPath::LeftEntityProvenance,
            Self::RightEntityConfidence => EntityQueryPath::RightEntityConfidence,
            Self::RightEntityProvenance => EntityQueryPath::RightEntityProvenance,
            Self::Provenance(path) => EntityQueryPath::Provenance(path.map(JsonPath::into_owned)),
            Self::EditionProvenance(path) => {
                EntityQueryPath::EditionProvenance(path.map(JsonPath::into_owned))
            }
            Self::PropertyMetadata(path) => {
                EntityQueryPath::PropertyMetadata(path.map(JsonPath::into_owned))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::iter::once;

    use super::*;

    #[test]
    fn sorting_path_deserialization_error() {
        assert_eq!(
            EntityQueryPath::deserialize_from_sorting_tokens(de::value::SeqDeserializer::<
                _,
                de::value::Error,
            >::new(once("invalid")))
            .expect_err("managed to convert entity query sorting path")
            .to_string(),
            format!(
                "unknown variant `invalid`, expected {}",
                EntityQuerySortingVisitor::EXPECTING
            )
        );
    }

    fn deserialize<'p>(segments: impl IntoIterator<Item = &'p str>) -> EntityQueryPath<'p> {
        EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
            segments.into_iter(),
        ))
        .expect("could not deserialize path")
    }

    #[test]
    fn deserialization() {
        assert_eq!(deserialize(["ownedById"]), EntityQueryPath::OwnedById);
        assert_eq!(
            deserialize(["type", "version"]),
            EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPath::Version,
                inheritance_depth: None,
            }
        );
        assert_eq!(
            deserialize(["type(inheritanceDepth = 5)", "version"]),
            EntityQueryPath::EntityTypeEdge {
                edge_kind: SharedEdgeKind::IsOfType,
                path: EntityTypeQueryPath::Version,
                inheritance_depth: Some(5),
            }
        );
        assert_eq!(
            deserialize([
                "properties",
                "https://blockprotocol.org/@alice/types/property-type/name/"
            ]),
            EntityQueryPath::Properties(Some(JsonPath::from_path_tokens(vec![PathToken::Field(
                Cow::Borrowed("https://blockprotocol.org/@alice/types/property-type/name/")
            )])))
        );
        assert_eq!(
            deserialize(["leftEntity", "uuid"]),
            EntityQueryPath::EntityEdge {
                edge_kind: KnowledgeGraphEdgeKind::HasLeftEntity,
                path: Box::new(EntityQueryPath::Uuid),
                direction: EdgeDirection::Outgoing
            }
        );

        assert_eq!(
            EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
                once("invalid")
            ))
            .expect_err(
                "managed to convert entity query path with hidden token when it should have \
                 errored"
            )
            .to_string(),
            format!(
                "unknown variant `invalid`, expected {}",
                EntityQueryPathVisitor::EXPECTING
            )
        );

        assert_eq!(
            EntityQueryPath::deserialize(de::value::SeqDeserializer::<_, de::value::Error>::new(
                ["editionId", "test"].into_iter()
            ))
            .expect_err(
                "managed to convert entity query path with multiple tokens when it should have \
                 errored"
            )
            .to_string(),
            "invalid length 2, expected 1 element in sequence"
        );
    }
}

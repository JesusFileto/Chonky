//! General types and traits used throughout the Zanzibar authorization system.

use core::fmt;
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// The relation or permission of a [`Subject`] to an [`Resource`].
pub trait Affiliation<R: Resource + ?Sized>: AsRef<str> {}

/// A computed set of [`Subject`]s for a particular [`Resource`].
pub trait Permission<R: Resource + ?Sized>: Affiliation<R> {}

/// Encapsulates the relationship between a [`Subject`] and a [`Resource`].
pub trait Relation<R: Resource + ?Sized>: Affiliation<R> {}

/// A [`Relation`] or [`Permission`] which is not tied to a specific [`Resource`].
///
/// This is useful for when the [`Resource`] type is not known at compile-time, e.g. when parsing a
/// [`Tuple`] from a string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenericAffiliation<A>(pub A);

impl<A> AsRef<str> for GenericAffiliation<A>
where
    A: AsRef<str>,
{
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<A: AsRef<str>, R: Resource + ?Sized> Affiliation<R> for GenericAffiliation<A> {}
impl<A: AsRef<str>, R: Resource + ?Sized> Permission<R> for GenericAffiliation<A> {}
impl<A: AsRef<str>, R: Resource + ?Sized> Relation<R> for GenericAffiliation<A> {}

/// Represent a unique entity that is being modelled.
///
/// `Resource`s are composed of a namespace and an unique identifier and often displayed as those
/// two values separated by a colon.
pub trait Resource {
    /// The unique identifier for this `Resource`.
    type Id: AsRef<str> + ?Sized;
    /// The namespace for this `Resource`.
    ///
    /// In most cases, this will be a static string.
    type Namespace: AsRef<str> + ?Sized;

    /// Returns the namespace for this `Resource`.
    fn namespace(&self) -> &Self::Namespace;

    /// Returns the unique identifier for this `Resource`.
    fn id(&self) -> &Self::Id;
}

/// A [`Resource`] that is generic over the [`Namespace`] and [`Id`] types.
///
/// This is useful for when the [`Namespace`] and [`Id`] types are not known at compile-time, e.g.
/// when parsing a [`Tuple`] from a string.
///
/// [`Namespace`]: Resource::Namespace
/// [`Id`]: Resource::Id
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GenericResource<N, I> {
    pub namespace: N,
    pub id: I,
}
impl<N, I> Resource for GenericResource<N, I>
where
    N: AsRef<str>,
    I: AsRef<str>,
{
    type Id = I;
    type Namespace = N;

    fn namespace(&self) -> &Self::Namespace {
        &self.namespace
    }

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

/// Represents either a [`Resource`] or combination of a [`Resource`] and [`Affiliation`].
pub trait Subject {
    /// The underlying [`Resource`] type for this `Subject`.
    type Resource: Resource + ?Sized;

    /// The relation to indicate that all [`Subject`]s found within the relation are to be
    /// included in the parent relation.
    type Affiliation: Affiliation<Self::Resource> + ?Sized;

    /// Returns the [`Resource`] for this `Subject`.
    fn resource(&self) -> &Self::Resource;

    /// Returns the [`Affiliation`] for this `Subject` if it exists.
    fn affiliation(&self) -> Option<&Self::Affiliation>;
}

impl<R> Subject for R
where
    R: Resource,
{
    type Resource = Self;

    type Affiliation = impl Affiliation<R>;

    fn resource(&self) -> &Self::Resource {
        self
    }

    fn affiliation(&self) -> Option<&Self::Affiliation> {
        struct Never;

        impl AsRef<str> for Never {
            fn as_ref(&self) -> &str {
                unreachable!()
            }
        }

        impl<R: Resource> Affiliation<R> for Never {}

        None::<Never>.as_ref()
    }
}

impl<R, A> Subject for (R, A)
where
    R: Resource,
    A: Affiliation<R>,
{
    type Affiliation = A;
    type Resource = R;

    fn resource(&self) -> &Self::Resource {
        &self.0
    }

    fn affiliation(&self) -> Option<&Self::Affiliation> {
        Some(&self.1)
    }
}

/// A [`Subject`] that is generic over the [`Resource`] and [`Affiliation`] types.
///
/// This is useful for when the [`Resource`] and [`Affiliation`] types are not known at
/// compile-time, e.g. when parsing a [`Tuple`] from a string.
///
/// [`Resource`]: Subject::Resource
/// [`Affiliation`]: Subject::Affiliation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GenericSubject<R, A> {
    pub resource: R,
    pub affiliation: Option<A>,
}

impl<R: Resource, A: Affiliation<R>> Subject for GenericSubject<R, A> {
    type Affiliation = A;
    type Resource = R;

    fn resource(&self) -> &Self::Resource {
        &self.resource
    }

    fn affiliation(&self) -> Option<&Self::Affiliation> {
        self.affiliation.as_ref()
    }
}

/// Represent the existence of a live relation between a [`Resource`] and [`Subject`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple<R, A, S> {
    pub resource: R,
    pub affiliation: A,
    pub subject: S,
}

impl<R, A, S> fmt::Display for Tuple<R, A, S>
where
    R: Resource,
    A: Affiliation<R>,
    S: Subject,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}:{}#{}@{}:{}",
            self.resource.namespace().as_ref(),
            self.resource.id().as_ref(),
            self.affiliation.as_ref(),
            self.subject.resource().namespace().as_ref(),
            self.subject.resource().id().as_ref()
        )?;
        if let Some(affiliation) = self.subject.affiliation() {
            write!(fmt, "#{}", affiliation.as_ref())?;
        }
        Ok(())
    }
}

/// An untyped [`Tuple`] that is generic over the [`Resource`], [`Affiliation`], and [`Subject`].
///
/// This is useful for when the [`Resource`], [`Affiliation`], and [`Subject`] types are not known
/// at compile-time, e.g. when parsing a [`Tuple`] from a string.
pub type StringTuple = Tuple<
    GenericResource<String, String>,
    GenericAffiliation<String>,
    GenericSubject<GenericResource<String, String>, GenericAffiliation<String>>,
>;

impl StringTuple {
    #[must_use]
    pub fn from_tuple<R, A, S>(resource: &R, affiliation: &A, subject: &S) -> Self
    where
        R: Resource + ?Sized,
        A: Affiliation<R> + ?Sized,
        S: Subject + ?Sized,
    {
        Self {
            resource: GenericResource {
                namespace: resource.namespace().as_ref().to_owned(),
                id: resource.id().as_ref().to_owned(),
            },
            affiliation: GenericAffiliation(affiliation.as_ref().to_owned()),
            subject: GenericSubject {
                resource: GenericResource {
                    namespace: subject.resource().namespace().as_ref().to_owned(),
                    id: subject.resource().id().as_ref().to_owned(),
                },
                affiliation: subject
                    .affiliation()
                    .map(|affiliation| GenericAffiliation(affiliation.as_ref().to_owned())),
            },
        }
    }
}

/// Provide causality metadata between Write and Check requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Zookie<'a>(Cow<'a, str>);

/// Specifies the desired consistency level on a per-request basis.
///
/// This allows for the API consumers dynamically trade-off less fresh data for more performance
/// when possible.
#[derive(Debug)]
pub enum Consistency<'z> {
    /// Attempts to minimize the latency of the API call, using whatever caches are available.
    ///
    /// > ## Warning
    /// >
    /// > If used exclusively, this can lead to a window of time where the New Enemy Problem can
    /// > occur.
    MinimalLatency,
    /// Ensures that all data used for computing the response is at least as fresh as the
    /// point-in-time specified in the [`Zookie`].
    ///
    /// If newer information is available, it will be used.
    AtLeastAsFresh(Zookie<'z>),
    /// Ensures that all data used for computing the response is that found at the exact
    /// point-in-time specified in the [`Zookie`].
    ///
    /// If the snapshot is not available, an error will be raised.
    AtExactSnapshot(Zookie<'z>),
    /// Ensure that all data used is fully consistent with the latest data available within the
    /// SpiceDB datastore.
    ///
    /// Note that the snapshot used will be loaded at the beginning of the API call, and that new
    /// data written after the API starts executing will be ignored.
    ///
    /// > ## Warning
    /// >
    /// > Use of `FullyConsistent` means little caching will be available, which means performance
    /// > will suffer. Only use if a [`Zookie`] is not available or absolutely latest information
    /// > is required.
    FullyConsistent,
}
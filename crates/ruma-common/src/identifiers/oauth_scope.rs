//! An arbitrary, opaque OAuth scope.
//! 
//! This type is distinct from [`ruma_common::api::OAuthClientScope`], which enumerates
//! OAuth scopes that have meaning in the Matrix specification.

use ruma_macros::IdDst;

use crate::api::OAuthClientScope;

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
#[ruma_id(validate = ruma_identifiers_validation::oauth_scope::validate)]
pub struct OAuthScope(str);


//! Single Sign-On (SSO) Support
//!
//! SAML 2.0 and SCIM 2.0 for enterprise identity integration.

pub mod saml;

pub use saml::{SamlManager, SamlIdentityProvider, SsoUser, SamlResponse, SamlStatus};
pub use saml::scim::{ScimProvisioner, ScimUser};

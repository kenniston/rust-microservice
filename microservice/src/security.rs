pub mod oauth2 {
    use std::collections::{HashMap, HashSet};

    use colored::Colorize;
    use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use tracing::warn;

    use crate::settings::Settings;

    /// A type alias for a `Result` with the `ServerError` error type.
    pub type Result<T, E = OAuth2Error> = std::result::Result<T, E>;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Token {
        pub access_token: Option<String>,
        pub expires_in: Option<u64>,
        pub refresh_expires_in: Option<u64>,
        pub refresh_token: Option<String>,
        pub token_type: Option<String>,
        pub id_token: Option<String>,
        pub session_state: Option<String>,
        pub scope: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct LoginForm {
        pub grant_type: String,
        pub username: Option<String>,
        pub password: Option<String>,
        pub client_id: Option<String>,
        pub client_secret: Option<String>,
        pub scope: Option<String>,
    }

    impl LoginForm {
        pub fn to_urlencoded(&self) -> String {
            let mut urlencoded = String::new();
            urlencoded.push_str("grant_type=");
            urlencoded.push_str(&self.grant_type);
            urlencoded.push_str("&username=");
            urlencoded.push_str(self.username.as_ref().unwrap_or(&String::new()));
            urlencoded.push_str("&password=");
            urlencoded.push_str(self.password.as_ref().unwrap_or(&String::new()));
            urlencoded.push_str("&client_id=");
            urlencoded.push_str(self.client_id.as_ref().unwrap_or(&String::new()));
            urlencoded.push_str("&client_secret=");
            urlencoded.push_str(self.client_secret.as_ref().unwrap_or(&String::new()));
            urlencoded.push_str("&scope=");
            urlencoded.push_str(self.scope.as_ref().unwrap_or(&String::new()));
            urlencoded
        }
    }

    #[derive(Debug, Error)]
    pub enum OAuth2Error {
        #[error("Invalid OAuth2 configuration: {0}")]
        Configuration(String),

        #[error("Invalid JWT token: {0}")]
        InvalidJwt(String),

        #[error("Invalid server public key: {0}")]
        InvalidPublicKey(String),

        #[error("JWT Decode error: {0}")]
        JWTDecode(String),

        #[error("Unauthorized: {0}")]
        Unauthorized(String),

        #[error("Error parsing authorization: {0}")]
        RoleAuthorizationParse(String),

        #[error("Invalid roles: {0}")]
        InvalidRoles(String),
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        // Optional. Audience
        aud: Option<String>,

        // Required (validate_exp defaults to true in validation).
        // Expiration time (as UTC timestamp)
        exp: Option<usize>,

        // Optional. Issued at (as UTC timestamp)
        iat: Option<usize>,

        // Optional. Issuer
        iss: Option<String>,

        // Optional. Not Before (as UTC timestamp)
        nbf: Option<usize>,

        // Optional. Subject (whom token refers to)
        sub: Option<String>,

        // Optional. Auth Scopes
        scope: Option<String>,

        // Optional. Resource Access
        resource_access: Option<HashMap<String, Realm>>,
    }

    impl Claims {
        /// Returns an optional HashSet of roles if the JWT token contains a
        /// "resource_access" claim.
        ///
        /// The roles are extracted from the "resource_access" claim in the
        /// JWT token, which is a map of resource names to Realm objects.
        /// The roles are then flattened and collected into a HashSet.
        ///
        /// If the JWT token does not contain a "resource_access" claim, or
        /// if the claim is empty, an empty Option is returned.
        pub fn get_roles(&self) -> Option<HashSet<String>> {
            Some(
                self.resource_access
                    .as_ref()?
                    .values()
                    .flat_map(|v| v.roles.clone())
                    .map(|role| {
                        format!(
                            "ROLE_{}",
                            role.to_uppercase().replace("-", "_").replace(" ", "_")
                        )
                    })
                    .collect(),
            )
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Realm {
        // Optional Additional claims
        roles: Vec<String>,
    }

    /// Validates a JWT token and ensures that the roles in the token match the provided list.
    ///
    /// # Parameters
    /// - `token`: The JWT token to validate.
    /// - `settings`: The configuration settings for the server.
    /// - `roles`: The list of roles to check against the JWT token.
    ///
    /// # Returns
    /// A `Result` containing a `()`` if the JWT token is valid and the roles match.
    /// Returns an error if the JWT token is invalid or the roles do not match.
    ///
    /// # Errors
    /// This method will return an error if:
    /// - The JWT token is invalid.
    /// - The roles in the JWT token do not match the provided list.
    pub fn validate_jwt(token: &str, settings: &Settings, authorize: String) -> Result<()> {
        // Validate JWT and retrieve the `kid` header
        let (kid, algorithm) = validate_jwt_header(token)?;

        validate_jwt_with_roles(token, kid.as_str(), algorithm, authorize, settings)?;

        Ok(())
    }

    /// Validates the JWT header and retrieves the `kid` and `Algorithm` fields.
    ///
    /// # Parameters
    /// - `token`: The JWT token to validate and extract the header fields from.
    ///
    /// # Returns
    /// A `Result` containing a tuple of `(String, Algorithm)` if the header is valid.
    /// Returns an error if the header is invalid or if the `kid` field is not present.
    ///
    /// # Errors
    /// This method will return an error if:
    /// - The JWT header is invalid.
    /// - The `kid` field is not present in the JWT header.
    pub fn validate_jwt_header(token: &str) -> Result<(String, Algorithm)> {
        let header = decode_header(token)
            .map_err(|_| OAuth2Error::InvalidJwt("Invalid JWT Header.".into()))?;

        let Some(kid) = header.kid else {
            warn!("Token doesn't have a `kid` header field.");
            return Err(OAuth2Error::InvalidJwt(
                "Token doesn't have a `kid` header field.".into(),
            ));
        };

        Ok((kid, header.alg))
    }

    /// Validates a JWT token and ensures that the roles in the token match the provided list.
    ///
    /// This method takes in a JWT token, a kid, an algorithm, a list of roles, and a settings object.
    /// It first retrieves the public key based on the `kid` and then uses it to decode the JWT token.
    /// After decoding, it validates that the issuer URI within the token matches the one configured in the server settings.
    /// Finally, it checks that the roles in the token match the provided list.
    ///
    /// # Parameters
    /// - `token`: The JWT token to validate.
    /// - `kid`: The key id to retrieve the public key for.
    /// - `algorithm`: The algorithm used to decode the JWT token.
    /// - `roles`: The list of roles to check against the JWT token.
    /// - `settings`: The settings object containing the server configuration.
    ///
    /// # Returns
    /// A `Result` containing a `()` if the JWT token is valid and the roles match.
    /// Returns an error if the JWT token is invalid or the roles do not match.
    ///
    /// # Errors
    /// This method will return an error if:
    /// - The JWT token is invalid.
    /// - The roles in the JWT token do not match the provided list.
    /// - The public key is not found for the given `kid`.
    /// - The issuer URI is not configured in the server settings.
    pub fn validate_jwt_with_roles(
        token: &str,
        kid: &str,
        algorithm: Algorithm,
        authorize: String,
        settings: &Settings,
    ) -> Result<()> {
        // Retrieves the public key based on the `kid`
        let public_key = settings.get_auth2_public_key(kid).ok_or_else(|| {
            warn!("Public key not found for key id: {kid}.");
            OAuth2Error::InvalidPublicKey("Public key not found for key id: {kid}.".into())
        })?;
        let decoded_public_key = &DecodingKey::try_from(&public_key).map_err(|e| {
            warn!("Invalid public key. \n{:?}", &public_key);
            OAuth2Error::InvalidPublicKey(e.to_string())
        })?;

        // Retrieves the issuer URI within the server configuration
        let issuer = settings
            .get_oauth2_config()
            .ok_or_else(|| {
                warn!("Security not configured.");
                OAuth2Error::Configuration("Security not configured..".into())
            })?
            .issuer_uri
            .ok_or_else(|| {
                warn!("Issuer URI not configured.");
                OAuth2Error::Configuration("Issuer URI not configured.".into())
            })?;

        // Creates a validation struct for the JWT
        let validation = {
            let mut validation = Validation::new(algorithm);
            validation.set_issuer(&[issuer.as_str()]);
            validation.validate_exp = true;
            validation
        };

        // Decodes the JWT into a HashMap
        let decoded_token =
            decode::<Claims>(token, decoded_public_key, &validation).map_err(|e| {
                warn!("Invalid token. {}", e.to_string());
                OAuth2Error::JWTDecode(e.to_string())
            })?;

        validate_jwt_roles(&decoded_token, authorize)?;

        Ok(())
    }

    /// Validates the roles in the given JWT token against the provided authorization string.
    ///
    /// The `authorize` string must be in the following format:
    /// `method role1,role2,...,roleN` or `ROLE1,ROLE2,...,ROLEN`
    ///
    /// The `method` parameter can be either "hasanyrole" or "hasallroles".
    /// If "hasanyrole" is specified, the function will return an error if any of the required roles are not found in the JWT token.
    /// If "hasallroles" is specified, the function will return an error if all of the required roles are not found in the JWT token.
    ///
    /// # Parameters
    /// - `token`: The JWT token to validate the roles against.
    /// - `authorize`: The authorization string to parse.
    ///
    /// # Returns
    /// A `Result` containing a unit if the roles match the provided authorization string.
    /// Returns an error if the roles do not match the provided authorization string.
    ///
    /// # Errors
    /// This method will return an error if:
    /// - The authorization string is invalid.
    /// - The roles in the JWT token do not match the provided authorization string.
    fn validate_jwt_roles(token: &TokenData<Claims>, authorize: String) -> Result<()> {
        let (method, roles) = get_authorize_role_method(authorize)?;

        match method.as_str() {
            "hasanyrole" => has_any_role(token, roles)?,
            "hasallroles" => has_all_role(token, roles)?,
            _ => {
                if !method.is_empty() {
                    return Err(OAuth2Error::InvalidRoles(format!(
                        "Invalid role authorization method: {}",
                        method.bright_blue()
                    )));
                } else {
                    // Validate Single Role
                    has_any_role(token, roles)?;
                }
            }
        }

        Ok(())
    }

    /// Checks if all of the roles in the given `roles` vector are present in the JWT token.
    ///
    /// # Parameters
    /// - `token`: The JWT token to check the roles against.
    /// - `roles`: A vector of roles to check against the JWT token.
    ///
    /// # Returns
    /// A `Result` containing a unit if all of the required roles are found in the JWT token.
    /// Returns an error if any of the required roles are not found in the JWT token.
    ///
    /// # Errors
    /// This method will return an error if none of the required roles are found in the JWT token.
    fn has_all_role(token: &TokenData<Claims>, roles: Vec<String>) -> Result<()> {
        let token_roles = token
            .claims
            .get_roles()
            .ok_or_else(|| OAuth2Error::InvalidRoles("User doesn't have any roles.".into()))?;
        for role in &roles {
            if !token_roles.contains(role) {
                return Err(OAuth2Error::InvalidRoles(format!(
                    "No required role was found for the current user. Required roles: {}. Current roles: {}",
                    role.bright_blue(),
                    token_roles
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                        .bright_green()
                )));
            }
        }

        Ok(())
    }

    /// Checks if any of the roles in the given `roles` vector is present in the JWT token.
    ///
    /// # Parameters
    /// - `token`: The JWT token to check the roles against.
    /// - `roles`: A vector of roles to check against the JWT token.
    ///
    /// # Returns
    /// A `Result` containing a unit if any of the required roles are found in the JWT token.
    /// Returns an error if none of the required roles are found in the JWT token.
    ///
    /// # Errors
    /// This method will return an error if none of the required roles are found in the JWT token.
    fn has_any_role(token: &TokenData<Claims>, roles: Vec<String>) -> Result<()> {
        let token_roles = token
            .claims
            .get_roles()
            .ok_or_else(|| OAuth2Error::InvalidRoles("User doesn't have any roles.".into()))?;
        for role in &roles {
            if token_roles.contains(role) {
                return Ok(());
            }
        }

        Err(OAuth2Error::InvalidRoles(format!(
            "No required role was found for the current user. Required roles: {}. Current roles: {}",
            roles.join(", ").bright_blue(),
            token_roles
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .bright_green()
        )))
    }

    /// Retrieves the authorization method and roles from the given string.
    ///
    /// The authorization string must be in the following format:
    /// `method role1,role2,...,roleN` or `ROLE1,ROLE2,...,ROLEN`
    ///
    /// # Parameters
    /// - `authorize`: The authorization string to parse.
    ///
    /// # Returns
    /// A `Result` containing a tuple of the authorization method and roles if the string is valid.
    /// Returns an error if the string is invalid.
    ///
    /// # Errors
    /// This method will return an error if the authorization string is invalid.
    fn get_authorize_role_method(authorize: String) -> Result<(String, Vec<String>)> {
        let pattern = Regex::new(
            r"(?i)^\s*(?:(\w+)\s*\(\s*(ROLE_\w+(?:\s*,\s*ROLE_\w+)*)\s*\)|(ROLE_\w+))\s*$",
        )
        .map_err(|e| OAuth2Error::RoleAuthorizationParse(e.to_string()))?;

        let caps = pattern.captures(&authorize).ok_or_else(|| {
            OAuth2Error::RoleAuthorizationParse("Invalid role authorization format.".into())
        })?;

        // Grup 1: method (opcional)
        let method = caps
            .get(1)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_default();

        // Grup 2: roles with method
        // Grup 3: one role without method
        let roles_raw = caps
            .get(2)
            .or_else(|| caps.get(3))
            .map(|r| r.as_str())
            .ok_or_else(|| OAuth2Error::RoleAuthorizationParse("Roles not found.".into()))?;

        let roles = roles_raw
            .split(',')
            .map(|r| r.trim().to_uppercase())
            .collect::<Vec<_>>();

        // Method and roles cannot be empty at the same time
        if method.is_empty() && roles.is_empty() {
            return Err(OAuth2Error::RoleAuthorizationParse(
                "Authorization method and role not found.".into(),
            ));
        }

        // Roles cannot be empty if the method is not empty
        if !method.is_empty() && roles.is_empty() {
            return Err(OAuth2Error::RoleAuthorizationParse(
                "Authorization method without roles.".into(),
            ));
        }

        Ok((method, roles))
    }
}

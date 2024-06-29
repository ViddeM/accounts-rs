use std::{collections::HashSet, fmt::Display};

use serde::Serialize;

#[derive(Serialize)]
pub struct ScopeField {
    scope: String,
    fields: Vec<String>,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, PartialEq, Eq, Hash)]
#[sqlx(type_name = "OAUTH_SCOPE", rename_all = "snake_case")]
pub enum OauthScope {
    Email,
    #[sqlx(rename = "openid")]
    OpenId,
}

impl OauthScope {
    pub fn get_scope_field(&self) -> ScopeField {
        match self {
            OauthScope::Email => ScopeField {
                scope: "email".into(),
                fields: vec!["Email Address".into()],
            },
            OauthScope::OpenId => ScopeField {
                scope: "openid".into(),
                fields: vec!["TODO".into()], // TODO: List fields here.
            },
        }
    }
}

impl TryFrom<&str> for OauthScope {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let scope = match value {
            "email" => OauthScope::Email,
            "openid" => OauthScope::OpenId,
            s => {
                log::warn!("Unsupported scope {s}");
                return Err(format!("Unsupported scope {s}"));
            }
        };

        Ok(scope)
    }
}

impl Display for OauthScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OauthScope::Email => "email",
                OauthScope::OpenId => "openid",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct OauthScopes {
    pub scopes: HashSet<OauthScope>,
}

impl OauthScopes {
    pub fn parse_or_default(scope: &Option<String>) -> Result<Self, String> {
        if let Some(s) = scope {
            Self::parse(s)
        } else {
            // Default to openid scope.
            Ok(Self {
                scopes: HashSet::from([OauthScope::OpenId]),
            })
        }
    }

    pub fn parse(scope: &String) -> Result<Self, String> {
        let scopes = scope
            .split(" ")
            .map(|s| OauthScope::try_from(s))
            .collect::<Result<HashSet<OauthScope>, String>>()?;

        Ok(OauthScopes { scopes })
    }
}

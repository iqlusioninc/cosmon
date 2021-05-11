//! Network IDs

pub use std::fmt::{self, Display};

/// Network IDs
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Id(String);

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<tendermint::chain::Id> for Id {
    fn from(chain_id: tendermint::chain::Id) -> Id {
        Id::from(&chain_id)
    }
}

impl From<&tendermint::chain::Id> for Id {
    fn from(chain_id: &tendermint::chain::Id) -> Id {
        Id(chain_id.as_str().to_owned())
    }
}

impl From<String> for Id {
    fn from(chain_id: String) -> Id {
        Id(chain_id)
    }
}

use super::*;

impl RelationshipLevel {
    /// How the dossier names this standing.
    pub fn label(self) -> &'static str {
        match self {
            Self::Hostile => "Hostile",
            Self::Neutral => "Neutral",
            Self::Friendly => "Friendly",
            Self::Trusted => "Trusted",
        }
    }
}

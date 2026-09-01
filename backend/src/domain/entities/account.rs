use uuid::Uuid;

/// Un compte auto-inscrit, consommateur ou marchand.
///
/// Distinct de [`super::ConsentSubject`] à dessein : celui-là désigne la
/// personne à qui se rapporte un consentement, celui-ci le compte dont on
/// change le mot de passe. Les deux circuits n'ont ni les mêmes règles ni la
/// même durée de vie, et les fondre ferait dépendre la réinitialisation des
/// évolutions du consentement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountSubject {
    Consumer(Uuid),
    Merchant(Uuid),
}

impl AccountSubject {
    /// Libellé du rôle, pour les journaux et les messages d'erreur.
    pub fn role(&self) -> &'static str {
        match self {
            Self::Consumer(_) => "consommateur",
            Self::Merchant(_) => "marchand",
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Self::Consumer(id) | Self::Merchant(id) => *id,
        }
    }
}

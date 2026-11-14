use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{resume_paniers, PickupValidationDto, ReservationConfirmationDto};
use crate::application::ports::{
    EventNotifier, NewReservationGroup, ProductRepository, RepoError, ReservationRepository,
    ReservationSummary,
};
use crate::domain::entities::ReservationStatus;
use crate::domain::services::reservation_code;

#[derive(Debug, Error)]
pub enum ReservationError {
    #[error("produit not found")]
    ProductNotFound,
    #[error("ce panier est épuisé, ou n'est plus disponible en quantité suffisante")]
    SoldOut,
    #[error("{0}")]
    InvalidInput(String),
    #[error("code de retrait introuvable")]
    ReservationNotFound,
    #[error("ce panier appartient à un autre marchand")]
    Forbidden,
    #[error("ce code a déjà été utilisé")]
    AlreadyRedeemed,
    #[error("impossible de générer un code de retrait unique, réessayez")]
    CodeGenerationFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const MAX_CODE_ATTEMPTS: u8 = 5;
/// Bornes d'une réservation groupée : assez pour un vrai panier de courses,
/// assez peu pour qu'un seul client ne vide pas un commerce d'un clic.
const MAX_LIGNES: usize = 20;
const MAX_QUANTITE: i32 = 10;
/// Paniers non retirés qu'un même client peut avoir chez un même commerce :
/// sans plafond, un seul client pourrait bloquer tout le stock d'un commerce
/// (les réservations ne s'annulent pas).
const MAX_EN_ATTENTE: i64 = 10;

/// Fusionne les doublons (le même panier demandé deux fois) et vérifie les
/// bornes. L'ordre de première apparition est gardé.
fn regrouper(demande: Vec<(Uuid, i32)>) -> Result<Vec<(Uuid, i32)>, ReservationError> {
    let mut lignes: Vec<(Uuid, i32)> = Vec::new();
    for (produit_id, quantite) in demande {
        if quantite < 1 {
            return Err(ReservationError::InvalidInput(
                "la quantité doit être d'au moins 1".into(),
            ));
        }
        match lignes.iter_mut().find(|(id, _)| *id == produit_id) {
            Some((_, total)) => *total = total.saturating_add(quantite),
            None => lignes.push((produit_id, quantite)),
        }
    }
    if lignes.is_empty() {
        return Err(ReservationError::InvalidInput(
            "choisissez au moins un panier".into(),
        ));
    }
    if lignes.len() > MAX_LIGNES {
        return Err(ReservationError::InvalidInput(format!(
            "{MAX_LIGNES} paniers différents au plus par réservation"
        )));
    }
    if lignes.iter().any(|(_, quantite)| *quantite > MAX_QUANTITE) {
        return Err(ReservationError::InvalidInput(format!(
            "{MAX_QUANTITE} exemplaires au plus d'un même panier"
        )));
    }
    Ok(lignes)
}

/// Reserving a panier and a marchand redeeming its pickup code in store.
pub struct ReservationUseCases {
    reservation_repo: Arc<dyn ReservationRepository>,
    product_repo: Arc<dyn ProductRepository>,
    event_notifier: Arc<dyn EventNotifier>,
}

impl ReservationUseCases {
    pub fn new(
        reservation_repo: Arc<dyn ReservationRepository>,
        product_repo: Arc<dyn ProductRepository>,
        event_notifier: Arc<dyn EventNotifier>,
    ) -> Self {
        Self {
            reservation_repo,
            product_repo,
            event_notifier,
        }
    }

    /// Réserve un seul panier : une réservation d'une ligne.
    pub async fn reserve(
        &self,
        consommateur_id: Uuid,
        produit_id: Uuid,
    ) -> Result<ReservationConfirmationDto, ReservationError> {
        self.reserve_several(consommateur_id, vec![(produit_id, 1)])
            .await
    }

    /// Réserve plusieurs paniers d'un même commerce en une fois, avec un seul
    /// code de retrait. Tout ou rien : si un panier n'est plus disponible en
    /// quantité suffisante, rien n'est réservé.
    pub async fn reserve_several(
        &self,
        consommateur_id: Uuid,
        demande: Vec<(Uuid, i32)>,
    ) -> Result<ReservationConfirmationDto, ReservationError> {
        let lignes = regrouper(demande)?;

        let mut offres = Vec::with_capacity(lignes.len());
        for (produit_id, _) in &lignes {
            let offre = self
                .product_repo
                .find_with_merchant(*produit_id)
                .await?
                .ok_or(ReservationError::ProductNotFound)?;
            offres.push(offre);
        }
        // Un seul code, donc un seul comptoir : un commerce par réservation.
        if offres
            .iter()
            .any(|offre| offre.marchand_id != offres[0].marchand_id)
        {
            return Err(ReservationError::InvalidInput(
                "une réservation ne peut regrouper que les paniers d'un même commerce".into(),
            ));
        }
        let deja = self
            .reservation_repo
            .paniers_en_attente(consommateur_id, offres[0].marchand_id)
            .await?;
        let demandes: i64 = lignes
            .iter()
            .map(|(_, quantite)| i64::from(*quantite))
            .sum();
        if deja + demandes > MAX_EN_ATTENTE {
            return Err(ReservationError::InvalidInput(format!(
                "{MAX_EN_ATTENTE} paniers au plus en attente de retrait chez un même commerce \
                 (vous en avez déjà {deja})"
            )));
        }

        let mut last_error = ReservationError::CodeGenerationFailed;
        for _ in 0..MAX_CODE_ATTEMPTS {
            let code = reservation_code::generate();
            match self
                .reservation_repo
                .create_group(NewReservationGroup {
                    consommateur_id,
                    code,
                    lignes: lignes.clone(),
                })
                .await
            {
                Ok(reservations) => {
                    let confirmation = ReservationConfirmationDto::from_group(
                        reservations.into_iter().zip(offres).collect(),
                    );
                    self.event_notifier
                        .notify(
                            "nouvelle_reservation",
                            format!(
                                "Nouvelle réservation : {} chez {} (code {})",
                                confirmation.produit_nom,
                                confirmation.marchand_nom,
                                confirmation.code
                            ),
                        )
                        .await;
                    return Ok(confirmation);
                }
                Err(RepoError::Conflict(_)) => continue,
                Err(RepoError::NotFound) => return Err(ReservationError::SoldOut),
                Err(other) => {
                    last_error = other.into();
                    break;
                }
            }
        }

        Err(last_error)
    }

    /// Le commerçant tape le code une fois : tous les paniers réservés sous
    /// ce code sont remis.
    pub async fn validate_pickup(
        &self,
        marchand_id: Uuid,
        code: &str,
    ) -> Result<PickupValidationDto, ReservationError> {
        let reservations = self.reservation_repo.find_by_code(code).await?;
        if reservations.is_empty() {
            return Err(ReservationError::ReservationNotFound);
        }

        let mut paniers = Vec::with_capacity(reservations.len());
        for reservation in &reservations {
            let product = self
                .product_repo
                .find_by_id(reservation.produit_id)
                .await?
                .ok_or(ReservationError::ProductNotFound)?;
            if product.marchand_id != marchand_id {
                return Err(ReservationError::Forbidden);
            }
            paniers.push((product.nom, reservation.quantite));
        }
        if reservations
            .iter()
            .any(|reservation| reservation.statut != ReservationStatus::Reservee)
        {
            return Err(ReservationError::AlreadyRedeemed);
        }

        self.reservation_repo.mark_recuperee_by_code(code).await?;
        let produit_nom = resume_paniers(paniers.iter().map(|(nom, q)| (nom.as_str(), *q)));
        let code = reservations[0].code.clone();
        self.event_notifier
            .notify(
                "panier_recupere",
                format!("Panier récupéré : {produit_nom} (code {code})"),
            )
            .await;
        Ok(PickupValidationDto { code, produit_nom })
    }

    pub async fn list_my_reservations(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<ReservationSummary>, ReservationError> {
        Ok(self
            .reservation_repo
            .list_by_consumer(consommateur_id)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_doublons_sont_fusionnes() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_eq!(
            regrouper(vec![(a, 1), (b, 2), (a, 2)]).unwrap(),
            vec![(a, 3), (b, 2)]
        );
    }

    #[test]
    fn une_demande_vide_ou_hors_bornes_est_refusee() {
        let a = Uuid::new_v4();
        assert!(regrouper(vec![]).is_err());
        assert!(regrouper(vec![(a, 0)]).is_err());
        assert!(regrouper(vec![(a, MAX_QUANTITE + 1)]).is_err());
        let trop: Vec<_> = (0..=MAX_LIGNES).map(|_| (Uuid::new_v4(), 1)).collect();
        assert!(regrouper(trop).is_err());
    }
}

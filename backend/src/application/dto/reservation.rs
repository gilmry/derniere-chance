use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::ProductWithMerchant;
use crate::domain::entities::{Reservation, ReservationStatus};

/// Un panier d'une réservation : son nom, combien, à quel prix unitaire.
#[derive(Debug, Serialize)]
pub struct LigneReservationDto {
    pub produit_id: Uuid,
    pub produit_nom: String,
    pub quantite: i32,
    pub prix_demarque: Decimal,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
}

/// What the consommateur sees on the "Panier réservé !" screen: the code to
/// present in store, plus enough merchant/produit context to render it
/// without a follow-up request. Une réservation peut regrouper plusieurs
/// paniers du même commerce (`lignes`) : `produit_nom` les résume
/// (« 2 × Panier A, Panier B »), `prix_demarque` est le total et la fenêtre
/// de retrait est celle où tous peuvent être retirés.
#[derive(Debug, Serialize)]
pub struct ReservationConfirmationDto {
    pub id: Uuid,
    pub code: String,
    pub statut: ReservationStatus,
    pub marchand_nom: String,
    pub produit_nom: String,
    pub prix_demarque: Decimal,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub lignes: Vec<LigneReservationDto>,
}

/// What the marchand sees after avoir validé un code en boutique.
/// `produit_nom` résume tous les paniers remis sous ce code.
#[derive(Debug, Serialize)]
pub struct PickupValidationDto {
    pub code: String,
    pub produit_nom: String,
}

/// « 2 × Panier A, Panier B » : la quantité n'est écrite qu'au-delà de 1.
pub fn resume_paniers<'a>(lignes: impl IntoIterator<Item = (&'a str, i32)>) -> String {
    lignes
        .into_iter()
        .map(|(nom, quantite)| {
            if quantite > 1 {
                format!("{quantite} × {nom}")
            } else {
                nom.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

impl ReservationConfirmationDto {
    /// `lignes` non vide, toutes sous le même code et chez le même commerce.
    pub fn from_group(lignes: Vec<(Reservation, ProductWithMerchant)>) -> Self {
        let (premiere, offre) = &lignes[0];
        let id = premiere.id;
        let code = premiere.code.clone();
        let statut = premiere.statut;
        let marchand_nom = offre.marchand_nom.clone();

        let produit_nom = resume_paniers(
            lignes
                .iter()
                .map(|(reservation, offre)| (offre.nom.as_str(), reservation.quantite)),
        );
        let prix_demarque = lignes
            .iter()
            .map(|(reservation, offre)| offre.prix_demarque * Decimal::from(reservation.quantite))
            .sum();

        // Fenêtre où tout peut être retiré : du plus tardif des débuts au plus
        // tôt des fins. Si elles ne se recouvrent pas, on montre la plus
        // large ; le détail par panier reste dans `lignes`.
        let debut_max = lignes
            .iter()
            .map(|(_, o)| o.retrait_debut)
            .max()
            .unwrap_or_default();
        let fin_min = lignes
            .iter()
            .map(|(_, o)| o.retrait_fin)
            .min()
            .unwrap_or_default();
        let (retrait_debut, retrait_fin) = if debut_max <= fin_min {
            (debut_max, fin_min)
        } else {
            (
                lignes
                    .iter()
                    .map(|(_, o)| o.retrait_debut)
                    .min()
                    .unwrap_or_default(),
                lignes
                    .iter()
                    .map(|(_, o)| o.retrait_fin)
                    .max()
                    .unwrap_or_default(),
            )
        };

        let lignes = lignes
            .into_iter()
            .map(|(reservation, offre)| LigneReservationDto {
                produit_id: offre.id,
                produit_nom: offre.nom,
                quantite: reservation.quantite,
                prix_demarque: offre.prix_demarque,
                retrait_debut: offre.retrait_debut,
                retrait_fin: offre.retrait_fin,
            })
            .collect();

        Self {
            id,
            code,
            statut,
            marchand_nom,
            produit_nom,
            prix_demarque,
            retrait_debut,
            retrait_fin,
            lignes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_resume_n_ecrit_la_quantite_qu_au_dela_de_un() {
        assert_eq!(
            resume_paniers([("Panier A", 2), ("Panier B", 1)]),
            "2 × Panier A, Panier B"
        );
    }
}

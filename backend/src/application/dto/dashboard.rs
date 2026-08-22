use rust_decimal::Decimal;
use serde::Serialize;

use crate::application::ports::{ConsumerStats, MerchantDailyStats};

#[derive(Debug, Serialize)]
pub struct MerchantDashboardDto {
    pub paniers_sauves: i64,
    pub chiffre_recupere: Decimal,
}

impl From<MerchantDailyStats> for MerchantDashboardDto {
    fn from(stats: MerchantDailyStats) -> Self {
        Self {
            paniers_sauves: stats.paniers_sauves,
            chiffre_recupere: stats.chiffre_recupere,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConsumerProfileDto {
    pub paniers_sauves: i64,
    pub montant_economise: Decimal,
}

impl From<ConsumerStats> for ConsumerProfileDto {
    fn from(stats: ConsumerStats) -> Self {
        Self {
            paniers_sauves: stats.paniers_sauves,
            montant_economise: stats.montant_economise,
        }
    }
}

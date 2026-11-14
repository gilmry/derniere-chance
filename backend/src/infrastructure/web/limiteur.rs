//! Limite de débit en mémoire, par client, pour les routes publiques qui
//! déclenchent un appel à un service extérieur (OpenStreetMap) : sans elle,
//! quelqu'un qui boucle sur ces routes remplirait la file d'attente des vrais
//! commerçants et pourrait faire bannir le serveur par le service.
//!
//! Volontairement simple (fenêtre fixe, une seule instance du backend). Le
//! client est identifié par l'adresse que donne le proxy (`X-Forwarded-For`
//! via Traefik) : un client qui la falsifie contourne la limite, mais pas la
//! file d'attente bornée des adaptateurs de géocodage, qui protège le service
//! dans tous les cas.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use actix_web::HttpRequest;

/// Au-delà, les entrées expirées sont purgées pour borner la mémoire.
const PURGE_AU_DELA: usize = 10_000;

pub struct Limiteur {
    max: u32,
    fenetre: Duration,
    compteurs: Mutex<HashMap<String, (Instant, u32)>>,
}

impl Limiteur {
    pub fn new(max: u32, fenetre: Duration) -> Self {
        Self {
            max,
            fenetre,
            compteurs: Mutex::new(HashMap::new()),
        }
    }

    /// `true` si le client `cle` peut encore faire une demande dans la
    /// fenêtre en cours (et la compte), `false` s'il a atteint la limite.
    pub fn autoriser(&self, cle: &str) -> bool {
        self.autoriser_a(cle, Instant::now())
    }

    fn autoriser_a(&self, cle: &str, maintenant: Instant) -> bool {
        let mut compteurs = self
            .compteurs
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if compteurs.len() > PURGE_AU_DELA {
            compteurs.retain(|_, (debut, _)| maintenant.duration_since(*debut) < self.fenetre);
        }
        let (debut, nombre) = compteurs.entry(cle.to_string()).or_insert((maintenant, 0));
        if maintenant.duration_since(*debut) >= self.fenetre {
            *debut = maintenant;
            *nombre = 0;
        }
        if *nombre >= self.max {
            return false;
        }
        *nombre += 1;
        true
    }
}

/// Suggestions d'adresses : la frappe en déclenche une toutes les 300 ms au
/// plus, une adresse complète en demande une dizaine.
/// `LIMITE_SUGGESTIONS_PAR_MINUTE` (30 par défaut).
pub fn suggestions() -> &'static Limiteur {
    static LIMITEUR: OnceLock<Limiteur> = OnceLock::new();
    LIMITEUR.get_or_init(|| {
        Limiteur::new(
            limite_env("LIMITE_SUGGESTIONS_PAR_MINUTE", 30),
            Duration::from_secs(60),
        )
    })
}

/// Inscriptions de commerçants : chacune vérifie l'adresse et le commerce
/// auprès de Nominatim (jusqu'à 4 requêtes).
/// `LIMITE_INSCRIPTIONS_PAR_HEURE` (10 par défaut ; relevée en dev et pour
/// les e2e, qui créent beaucoup de comptes depuis la même adresse).
pub fn inscriptions() -> &'static Limiteur {
    static LIMITEUR: OnceLock<Limiteur> = OnceLock::new();
    LIMITEUR.get_or_init(|| {
        Limiteur::new(
            limite_env("LIMITE_INSCRIPTIONS_PAR_HEURE", 10),
            Duration::from_secs(3600),
        )
    })
}

fn limite_env(cle: &str, defaut: u32) -> u32 {
    std::env::var(cle)
        .ok()
        .and_then(|valeur| valeur.trim().parse().ok())
        .filter(|limite| *limite > 0)
        .unwrap_or(defaut)
}

/// Adresse IP du client, sans le port.
pub fn cle_client(req: &HttpRequest) -> String {
    let info = req.connection_info();
    let adresse = info.realip_remote_addr().unwrap_or("inconnu");
    adresse
        .parse::<std::net::SocketAddr>()
        .map(|socket| socket.ip().to_string())
        .unwrap_or_else(|_| adresse.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloque_au_dela_de_la_limite_puis_repart_avec_une_nouvelle_fenetre() {
        let limiteur = Limiteur::new(2, Duration::from_secs(60));
        let t0 = Instant::now();
        assert!(limiteur.autoriser_a("1.2.3.4", t0));
        assert!(limiteur.autoriser_a("1.2.3.4", t0));
        assert!(!limiteur.autoriser_a("1.2.3.4", t0));
        // Un autre client n'est pas touché.
        assert!(limiteur.autoriser_a("5.6.7.8", t0));
        assert!(limiteur.autoriser_a("1.2.3.4", t0 + Duration::from_secs(61)));
    }
}

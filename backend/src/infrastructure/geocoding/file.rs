use std::sync::atomic::{AtomicUsize, Ordering};

/// Nombre de demandes en attente d'un service de géocodage, borné : les
/// adaptateurs envoient leurs requêtes une par une (politique d'usage
/// d'OpenStreetMap), donc une rafale de demandes ferait attendre tout le
/// monde de plus en plus longtemps. Au-delà de la borne, la demande est
/// refusée tout de suite (« service indisponible ») au lieu de s'empiler.
pub struct FileBornee {
    en_attente: AtomicUsize,
    max: usize,
}

/// Place réservée dans la file, rendue quand elle est abandonnée.
pub struct Place<'a>(&'a AtomicUsize);

impl Drop for Place<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl FileBornee {
    pub const fn new(max: usize) -> Self {
        Self {
            en_attente: AtomicUsize::new(0),
            max,
        }
    }

    /// `None` si la file est pleine.
    pub fn entrer(&self) -> Option<Place<'_>> {
        if self.en_attente.fetch_add(1, Ordering::SeqCst) >= self.max {
            self.en_attente.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Place(&self.en_attente))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuse_au_dela_de_la_borne_et_libere_la_place() {
        let file = FileBornee::new(1);
        let place = file.entrer();
        assert!(place.is_some());
        assert!(file.entrer().is_none());
        drop(place);
        assert!(file.entrer().is_some());
    }
}

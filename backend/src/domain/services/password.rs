/// Longueur minimale d'un mot de passe choisi par une personne.
///
/// Douze caractères plutôt que huit : la recommandation de l'ANSSI comme
/// celle du NIST privilégie la longueur sur la complexité imposée, et un
/// service qui n'a ni second facteur ni limitation de débit sur la connexion
/// n'a pas de quoi rattraper un mot de passe court.
pub const MIN_PASSWORD_LENGTH: usize = 12;

/// `Err` porte le message affiché à la personne, en français.
pub fn validate(password: &str) -> Result<(), String> {
    // Sur les caractères, pas sur les octets : « éé… » ne doit pas compter
    // double au seul motif que l'UTF-8 les encode sur deux octets.
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "le mot de passe doit faire au moins {MIN_PASSWORD_LENGTH} caractères"
        ));
    }
    if password.trim().is_empty() {
        return Err("le mot de passe ne peut pas être fait que d'espaces".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_mot_de_passe_trop_court_est_refuse() {
        assert!(validate("court").is_err());
        assert!(validate(&"a".repeat(MIN_PASSWORD_LENGTH - 1)).is_err());
    }

    #[test]
    fn la_longueur_se_compte_en_caracteres_pas_en_octets() {
        // 11 caractères accentués font 22 octets : les compter en octets
        // laisserait passer un mot de passe trop court.
        assert!(validate(&"é".repeat(MIN_PASSWORD_LENGTH - 1)).is_err());
        assert!(validate(&"é".repeat(MIN_PASSWORD_LENGTH)).is_ok());
    }

    #[test]
    fn un_mot_de_passe_d_espaces_est_refuse() {
        assert!(validate("                ").is_err());
    }

    #[test]
    fn un_mot_de_passe_suffisamment_long_passe() {
        assert!(validate("correct cheval batterie agrafe").is_ok());
    }
}

/// Pays où un commerce peut s'inscrire : les 27 pays de l'Union européenne,
/// plus la Suisse, le Royaume-Uni, la Norvège, l'Islande et le Liechtenstein.
/// Code ISO 3166-1 alpha-2 en majuscules, tel que stocké dans
/// `marchands.pays` et envoyé au géocodeur pour borner sa recherche.
///
/// La même liste vit côté frontend (`frontend/src/lib/pays.ts`) : les deux
/// doivent rester alignées.
pub const PAYS_SUPPORTES: &[(&str, &str)] = &[
    ("DE", "Allemagne"),
    ("AT", "Autriche"),
    ("BE", "Belgique"),
    ("BG", "Bulgarie"),
    ("CY", "Chypre"),
    ("HR", "Croatie"),
    ("DK", "Danemark"),
    ("ES", "Espagne"),
    ("EE", "Estonie"),
    ("FI", "Finlande"),
    ("FR", "France"),
    ("GR", "Grèce"),
    ("HU", "Hongrie"),
    ("IE", "Irlande"),
    ("IS", "Islande"),
    ("IT", "Italie"),
    ("LV", "Lettonie"),
    ("LI", "Liechtenstein"),
    ("LT", "Lituanie"),
    ("LU", "Luxembourg"),
    ("MT", "Malte"),
    ("NO", "Norvège"),
    ("NL", "Pays-Bas"),
    ("PL", "Pologne"),
    ("PT", "Portugal"),
    ("CZ", "République tchèque"),
    ("RO", "Roumanie"),
    ("GB", "Royaume-Uni"),
    ("SK", "Slovaquie"),
    ("SI", "Slovénie"),
    ("SE", "Suède"),
    ("CH", "Suisse"),
];

/// Pays retenu quand une requête n'en précise pas : celui des comptes créés
/// avant l'ajout du champ (voir la migration 0010).
pub const PAYS_PAR_DEFAUT: &str = "FR";

/// Normalise un code pays saisi (`" be "` → `"BE"`) et vérifie qu'il est dans
/// la liste ; renvoie le message à afficher sinon.
pub fn normaliser(code: &str) -> Result<String, String> {
    let code = code.trim().to_ascii_uppercase();
    if PAYS_SUPPORTES.iter().any(|(c, _)| *c == code) {
        Ok(code)
    } else {
        Err("pays non pris en charge : choisissez un pays européen dans la liste".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepte_un_code_en_minuscules_avec_espaces() {
        assert_eq!(normaliser(" be ").unwrap(), "BE");
    }

    #[test]
    fn refuse_un_pays_hors_europe() {
        assert!(normaliser("US").is_err());
        assert!(normaliser("").is_err());
    }

    #[test]
    fn la_liste_compte_32_pays_sans_doublon() {
        let mut codes: Vec<_> = PAYS_SUPPORTES.iter().map(|(c, _)| *c).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), 32);
    }

    #[test]
    fn le_pays_par_defaut_est_dans_la_liste() {
        assert!(normaliser(PAYS_PAR_DEFAUT).is_ok());
    }
}

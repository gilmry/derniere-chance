// Pays où un commerce peut s'inscrire : UE + Suisse, Royaume-Uni, Norvège,
// Islande, Liechtenstein. Même liste que le backend
// (backend/src/domain/services/pays.rs) : les deux doivent rester alignées.
export const PAYS: ReadonlyArray<readonly [code: string, nom: string]> = [
  ["DE", "Allemagne"],
  ["AT", "Autriche"],
  ["BE", "Belgique"],
  ["BG", "Bulgarie"],
  ["CY", "Chypre"],
  ["HR", "Croatie"],
  ["DK", "Danemark"],
  ["ES", "Espagne"],
  ["EE", "Estonie"],
  ["FI", "Finlande"],
  ["FR", "France"],
  ["GR", "Grèce"],
  ["HU", "Hongrie"],
  ["IE", "Irlande"],
  ["IS", "Islande"],
  ["IT", "Italie"],
  ["LV", "Lettonie"],
  ["LI", "Liechtenstein"],
  ["LT", "Lituanie"],
  ["LU", "Luxembourg"],
  ["MT", "Malte"],
  ["NO", "Norvège"],
  ["NL", "Pays-Bas"],
  ["PL", "Pologne"],
  ["PT", "Portugal"],
  ["CZ", "République tchèque"],
  ["RO", "Roumanie"],
  ["GB", "Royaume-Uni"],
  ["SK", "Slovaquie"],
  ["SI", "Slovénie"],
  ["SE", "Suède"],
  ["CH", "Suisse"],
];

export const PAYS_PAR_DEFAUT = "FR";

export function nomDuPays(code: string): string {
  return PAYS.find(([c]) => c === code)?.[1] ?? code;
}

// Client API pour le backend DernièreChance. Remplace les données mockées de
// mock.ts au fur et à mesure du branchement (voir VISION.md).

import type { Coords } from "./geoloc";
import {
  BETA_CONSENT_VERSION,
  CONSENT_PATH,
  MERCHANT_CONSENT_PATH,
  redirectToConsent,
} from "./consent";

const API_URL = import.meta.env.PUBLIC_API_URL ?? "http://localhost:8080";

// Catégories affichées comme filtres dans le feed - un marchand doit choisir
// l'une d'elles à l'inscription pour que le filtre par catégorie fonctionne
// (comparaison exacte côté backend, pas de texte libre reconnu).
export const MERCHANT_CATEGORIES: Record<string, string> = {
  Boulangerie: "🥐",
  Primeur: "🥕",
  Boucherie: "🥩",
  Épicerie: "🛒",
  Fleuriste: "💐",
};

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

/// Le backend a refusé l'appel faute de consentement bêta à jour (case
/// jamais cochée, ou consentement retiré depuis).
export class ConsentRequiredError extends ApiError {
  constructor(message: string) {
    super(403, message);
  }
}

async function apiFetch<T>(
  path: string,
  options: { method?: string; body?: unknown; token?: string | null } = {},
): Promise<T> {
  const headers: Record<string, string> = {};
  if (options.body !== undefined) headers["Content-Type"] = "application/json";
  if (options.token) headers["Authorization"] = `Bearer ${options.token}`;

  const res = await fetch(`${API_URL}${path}`, {
    method: options.method ?? "GET",
    headers,
    body: options.body !== undefined ? JSON.stringify(options.body) : undefined,
  });

  if (res.status === 204) return undefined as T;

  const isJson = res.headers.get("content-type")?.includes("application/json");
  const data = isJson ? await res.json() : undefined;

  if (!res.ok) {
    const message = (data && typeof data === "object" && "error" in data)
      ? String((data as { error: unknown }).error)
      : `Erreur ${res.status}`;

    // Tous les appels consommateur passent par ici : un seul endroit pour
    // rebasculer vers l'écran de consentement, plutôt que la même garde
    // recopiée dans chaque composant.
    const code = (data && typeof data === "object" && "code" in data)
      ? String((data as { code: unknown }).code)
      : undefined;
    if (res.status === 403 && code === "consentement_requis") {
      // Le chemin appelé dit de quel principal il s'agit, donc vers quel
      // écran renvoyer : les deux consentements sont distincts.
      redirectToConsent(path.startsWith("/marchands/") ? MERCHANT_CONSENT_PATH : CONSENT_PATH);
      throw new ConsentRequiredError(message);
    }

    throw new ApiError(res.status, message);
  }

  return data as T;
}

// --- Types (miroir des DTOs backend - rust_decimal sérialise en string) ---

export type ProductStatus = "publie" | "ecoule" | "expire";
export type ReservationStatus = "reservee" | "recuperee" | "expiree";

export interface Offer {
  id: string;
  marchand_id: string;
  marchand_nom: string;
  marchand_categorie: string;
  marchand_note: string | null;
  marchand_latitude: number | null;
  marchand_longitude: number | null;
  distance_km: number | null;
  nom: string;
  description: string;
  prix_initial: string;
  prix_demarque: string;
  reduction_pct: number;
  quantite: number;
  retrait_debut: string;
  retrait_fin: string;
  statut: ProductStatus;
  photo_url: string | null;
}

export interface Merchant {
  id: string;
  nom: string;
  adresse: string;
  categorie: string;
  note: string | null;
  latitude: number | null;
  longitude: number | null;
  logo_url: string | null;
  distance_km: number | null;
}

export interface Product {
  id: string;
  marchand_id: string;
  nom: string;
  description: string;
  prix_initial: string;
  prix_demarque: string;
  reduction_pct: number;
  quantite: number;
  retrait_debut: string;
  retrait_fin: string;
  statut: ProductStatus;
  photo_url: string | null;
}

export interface MerchantProfile extends Merchant {
  offres: Product[];
}

export interface ReservationConfirmation {
  id: string;
  code: string;
  statut: ReservationStatus;
  marchand_nom: string;
  produit_nom: string;
  prix_demarque: string;
  retrait_debut: string;
  retrait_fin: string;
}

export interface ReservationSummary {
  id: string;
  code: string;
  statut: ReservationStatus;
  marchand_nom: string;
  produit_nom: string;
  prix_demarque: string;
  retrait_debut: string;
  retrait_fin: string;
  created_at: string;
}

export interface PickupValidation {
  code: string;
  produit_nom: string;
}

export interface MerchantDashboard {
  paniers_sauves: number;
  chiffre_recupere: string;
}

export interface ConsumerProfile {
  paniers_sauves: number;
  montant_economise: string;
}

export interface AuthResponse {
  token: string;
}

// --- Catalogue public ---

function geoQueryParams(coords?: Coords | null): string {
  return coords ? `lat=${coords.lat}&lon=${coords.lon}` : "";
}

export function listOffers(categorie?: string, coords?: Coords | null): Promise<Offer[]> {
  const parts = [];
  if (categorie && categorie !== "Tout") parts.push(`categorie=${encodeURIComponent(categorie)}`);
  const geo = geoQueryParams(coords);
  if (geo) parts.push(geo);
  return apiFetch(`/offres${parts.length ? `?${parts.join("&")}` : ""}`);
}

export function getOffer(id: string, coords?: Coords | null): Promise<Offer> {
  const geo = geoQueryParams(coords);
  return apiFetch(`/offres/${id}${geo ? `?${geo}` : ""}`);
}

export function getMerchantProfile(id: string, coords?: Coords | null): Promise<MerchantProfile> {
  const geo = geoQueryParams(coords);
  return apiFetch(`/marchands/${id}${geo ? `?${geo}` : ""}`);
}

// --- Auth marchand ---

/// Comme pour un consommateur, l'inscription porte le consentement bêta : le
/// backend refuse la requête si la version envoyée n'est pas celle en
/// vigueur. Le marchand publie nom, adresse et position sur la carte, donc la
/// case cochée l'engage sur davantage.
export function merchantRegister(dto: {
  nom: string;
  adresse: string;
  categorie: string;
  email: string;
  password: string;
  latitude?: number | null;
  longitude?: number | null;
}): Promise<AuthResponse> {
  return apiFetch("/marchands/inscription", {
    method: "POST",
    body: { ...dto, consent_version: BETA_CONSENT_VERSION },
  });
}

export function merchantLogin(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/marchands/connexion", { method: "POST", body: { email, password } });
}

export function getMyMerchantProfile(token: string): Promise<Merchant> {
  return apiFetch("/marchands/moi", { token });
}

export function updateMerchantProfile(
  dto: { nom: string; adresse: string; categorie: string },
  token: string,
): Promise<Merchant> {
  return apiFetch("/marchands/moi", { method: "PATCH", body: dto, token });
}

// --- Auth consommateur ---

/// L'inscription porte le consentement bêta : le backend refuse la requête
/// si la version envoyée n'est pas celle en vigueur, donc aucun compte ne
/// peut naître sans consentement explicite et daté.
export function consumerRegister(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/consommateurs/inscription", {
    method: "POST",
    body: { email, password, consent_version: BETA_CONSENT_VERSION },
  });
}

export function consumerLogin(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/consommateurs/connexion", { method: "POST", body: { email, password } });
}

// --- Réinitialisation de mot de passe (non authentifié) ---

/// Longueur minimale imposée par le backend (domain/services/password.rs).
/// Répétée ici pour que le formulaire refuse avant l'aller-retour ; c'est le
/// backend qui fait foi.
export const MIN_PASSWORD_LENGTH = 12;

/// Répond toujours 204, que le compte existe ou non : le backend refuse de
/// dire qui est inscrit, et l'écran ne doit pas le déduire non plus.
/// La même route sert aux comptes clients et marchands.
export function forgotPassword(email: string): Promise<void> {
  return apiFetch("/mot-de-passe/oubli", { method: "POST", body: { email } });
}

/// `token` vient du lien reçu par email. Un 400 signale un lien invalide,
/// expiré, déjà utilisé, ou un mot de passe refusé.
export function resetPassword(token: string, password: string): Promise<void> {
  return apiFetch("/mot-de-passe/reinitialisation", {
    method: "POST",
    body: { token, password },
  });
}

// --- Consentement au programme bêta (auth: consommateur) ---

export interface ConsentStatus {
  /// Vrai seulement si le consentement porte sur `version_courante`.
  consenti: boolean;
  version_acceptee: string | null;
  accepte_le: string | null;
  version_courante: string;
}

/// Les deux principaux ont le même circuit de consentement, sur deux chemins
/// distincts parce que leurs jetons ne sont pas interchangeables.
export type ConsentRole = "consommateur" | "marchand";

function consentPath(role: ConsentRole): string {
  return role === "marchand"
    ? "/marchands/moi/consentement"
    : "/consommateurs/moi/consentement";
}

export function consentStatus(
  token: string,
  role: ConsentRole = "consommateur",
): Promise<ConsentStatus> {
  return apiFetch(consentPath(role), { token });
}

export function grantConsent(
  token: string,
  role: ConsentRole = "consommateur",
): Promise<ConsentStatus> {
  return apiFetch(consentPath(role), {
    method: "POST",
    body: { consent_version: BETA_CONSENT_VERSION },
    token,
  });
}

/// Retire le consentement et anonymise le compte côté serveur. Le jeton
/// local devient inutilisable : l'appelant doit le purger.
export function withdrawConsent(
  token: string,
  role: ConsentRole = "consommateur",
): Promise<void> {
  return apiFetch(consentPath(role), { method: "DELETE", token });
}

// --- Actions consommateur (auth requise) ---

export function reserveOffer(offerId: string, token: string): Promise<ReservationConfirmation> {
  return apiFetch(`/offres/${offerId}/reservation`, { method: "POST", token });
}

export function followMerchant(merchantId: string, token: string): Promise<void> {
  return apiFetch(`/marchands/${merchantId}/abonnement`, { method: "POST", token });
}

export function unfollowMerchant(merchantId: string, token: string): Promise<void> {
  return apiFetch(`/marchands/${merchantId}/abonnement`, { method: "DELETE", token });
}

export function listFollowedMerchants(token: string): Promise<Merchant[]> {
  return apiFetch("/consommateurs/moi/abonnements", { token });
}

export function consumerProfile(token: string): Promise<ConsumerProfile> {
  return apiFetch("/consommateurs/moi/profil", { token });
}

export function listMyReservations(token: string): Promise<ReservationSummary[]> {
  return apiFetch("/consommateurs/moi/reservations", { token });
}

// --- Backoffice marchand (auth requise) ---

export function listMyProducts(token: string): Promise<Product[]> {
  return apiFetch("/marchands/moi/produits", { token });
}

export function publishProduct(
  dto: {
    nom: string;
    description: string;
    prix_initial: string;
    prix_demarque: string;
    quantite: number;
    retrait_debut: string;
    retrait_fin: string;
    photo_url?: string | null;
  },
  token: string,
): Promise<Product> {
  return apiFetch("/marchands/moi/produits", { method: "POST", body: dto, token });
}

export function updateProduct(
  id: string,
  dto: {
    nom: string;
    description: string;
    prix_initial: string;
    prix_demarque: string;
    quantite: number;
    retrait_debut: string;
    retrait_fin: string;
    photo_url?: string | null;
  },
  token: string,
): Promise<Product> {
  return apiFetch(`/marchands/moi/produits/${id}`, { method: "PATCH", body: dto, token });
}

/// Upload une photo de panier, renvoie son URL publique à passer ensuite à
/// publishProduct(). Pas de JSON ici (multipart), donc pas apiFetch().
export async function uploadProductPhoto(file: File, token: string): Promise<string> {
  const form = new FormData();
  form.append("photo", file);

  const res = await fetch(`${API_URL}/marchands/moi/produits/photo`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: form,
  });

  const data = await res.json().catch(() => undefined);

  if (!res.ok) {
    const message = (data && typeof data === "object" && "error" in data)
      ? String((data as { error: unknown }).error)
      : `Erreur ${res.status}`;
    throw new ApiError(res.status, message);
  }

  return (data as { photo_url: string }).photo_url;
}

/// Upload le logo/photo du commerce, l'enregistre sur le compte marchand
/// (côté backend) et renvoie son URL publique.
export async function uploadMerchantLogo(file: File, token: string): Promise<string> {
  const form = new FormData();
  form.append("photo", file);

  const res = await fetch(`${API_URL}/marchands/moi/logo`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: form,
  });

  const data = await res.json().catch(() => undefined);

  if (!res.ok) {
    const message = (data && typeof data === "object" && "error" in data)
      ? String((data as { error: unknown }).error)
      : `Erreur ${res.status}`;
    throw new ApiError(res.status, message);
  }

  return (data as { logo_url: string }).logo_url;
}

export function markEcoule(productId: string, token: string): Promise<Product> {
  return apiFetch(`/marchands/moi/produits/${productId}/ecoule`, { method: "PATCH", token });
}

export function merchantDashboard(token: string): Promise<MerchantDashboard> {
  return apiFetch("/marchands/moi/dashboard", { token });
}

export function validatePickup(code: string, token: string): Promise<PickupValidation> {
  return apiFetch(`/marchands/moi/reservations/${code}/valider`, { method: "POST", token });
}

// --- Backoffice admin (auth: admin) ---

export interface AdminMerchant {
  id: string;
  nom: string;
  adresse: string;
  categorie: string;
  note: string | null;
  email: string;
  latitude: number | null;
  longitude: number | null;
  logo_url: string | null;
  created_at: string;
  /// Renseigné après retrait du consentement bêta : nom, adresse et position
  /// ne sont alors plus que des espaces réservés.
  anonymise_le: string | null;
}

export interface AdminConsumer {
  id: string;
  email: string;
  created_at: string;
  /// Renseigné après retrait du consentement bêta : `email` n'est alors
  /// plus qu'un identifiant technique.
  anonymise_le: string | null;
}

export interface AdminProduct extends Offer {
  created_at?: string;
}

export interface AdminStats {
  marchands: number;
  consommateurs: number;
  produits_actifs: number;
  reservations: number;
}

export function adminLogin(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/admin/connexion", { method: "POST", body: { email, password } });
}

export function adminListMerchants(token: string): Promise<AdminMerchant[]> {
  return apiFetch("/admin/marchands", { token });
}

export function adminDeleteMerchant(id: string, token: string): Promise<void> {
  return apiFetch(`/admin/marchands/${id}`, { method: "DELETE", token });
}

export function adminListConsumers(token: string): Promise<AdminConsumer[]> {
  return apiFetch("/admin/consommateurs", { token });
}

export function adminDeleteConsumer(id: string, token: string): Promise<void> {
  return apiFetch(`/admin/consommateurs/${id}`, { method: "DELETE", token });
}

export function adminListProducts(token: string): Promise<AdminProduct[]> {
  return apiFetch("/admin/produits", { token });
}

export function adminDeleteProduct(id: string, token: string): Promise<void> {
  return apiFetch(`/admin/produits/${id}`, { method: "DELETE", token });
}

export function adminUnpublishProduct(id: string, token: string): Promise<void> {
  return apiFetch(`/admin/produits/${id}/depublier`, { method: "PATCH", token });
}

export function adminStats(token: string): Promise<AdminStats> {
  return apiFetch("/admin/stats", { token });
}

// --- Formatage (rust_decimal arrive en string) ---

export function formatPrice(value: string | number): string {
  const n = typeof value === "string" ? parseFloat(value) : value;
  return `${n.toFixed(2).replace(".", ",")} €`;
}

export function formatDistance(km: number | null | undefined): string | null {
  if (km === null || km === undefined) return null;
  if (km < 1) return `${Math.round(km * 1000)} m`;
  return `${km.toFixed(1).replace(".", ",")} km`;
}

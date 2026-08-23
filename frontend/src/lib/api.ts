// Client API pour le backend DernièreChance. Remplace les données mockées de
// mock.ts au fur et à mesure du branchement (voir VISION.md).

import type { Coords } from "./geoloc";

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

export function merchantRegister(dto: {
  nom: string;
  adresse: string;
  categorie: string;
  email: string;
  password: string;
  latitude?: number | null;
  longitude?: number | null;
}): Promise<AuthResponse> {
  return apiFetch("/marchands/inscription", { method: "POST", body: dto });
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

export function consumerRegister(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/consommateurs/inscription", { method: "POST", body: { email, password } });
}

export function consumerLogin(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/consommateurs/connexion", { method: "POST", body: { email, password } });
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
}

export interface AdminConsumer {
  id: string;
  email: string;
  created_at: string;
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

// Client API pour le backend DernièreChance. Remplace les données mockées de
// mock.ts au fur et à mesure du branchement (voir VISION.md).

const API_URL = import.meta.env.PUBLIC_API_URL ?? "http://localhost:8080";

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
  nom: string;
  description: string;
  prix_initial: string;
  prix_demarque: string;
  reduction_pct: number;
  quantite: number;
  retrait_debut: string;
  retrait_fin: string;
  statut: ProductStatus;
}

export interface Merchant {
  id: string;
  nom: string;
  adresse: string;
  categorie: string;
  note: string | null;
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

export function listOffers(categorie?: string): Promise<Offer[]> {
  const qs = categorie && categorie !== "Tout" ? `?categorie=${encodeURIComponent(categorie)}` : "";
  return apiFetch(`/offres${qs}`);
}

export function getOffer(id: string): Promise<Offer> {
  return apiFetch(`/offres/${id}`);
}

export function getMerchantProfile(id: string): Promise<MerchantProfile> {
  return apiFetch(`/marchands/${id}`);
}

// --- Auth marchand ---

export function merchantRegister(dto: {
  nom: string;
  adresse: string;
  categorie: string;
  email: string;
  password: string;
}): Promise<AuthResponse> {
  return apiFetch("/marchands/inscription", { method: "POST", body: dto });
}

export function merchantLogin(email: string, password: string): Promise<AuthResponse> {
  return apiFetch("/marchands/connexion", { method: "POST", body: { email, password } });
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
  },
  token: string,
): Promise<Product> {
  return apiFetch("/marchands/moi/produits", { method: "POST", body: dto, token });
}

export function markEcoule(productId: string, token: string): Promise<Product> {
  return apiFetch(`/marchands/moi/produits/${productId}/ecoule`, { method: "PATCH", token });
}

export function merchantDashboard(token: string): Promise<MerchantDashboard> {
  return apiFetch("/marchands/moi/dashboard", { token });
}

export function validatePickup(code: string, token: string): Promise<void> {
  return apiFetch(`/marchands/moi/reservations/${code}/valider`, { method: "POST", token });
}

// --- Formatage (rust_decimal arrive en string) ---

export function formatPrice(value: string | number): string {
  const n = typeof value === "string" ? parseFloat(value) : value;
  return `${n.toFixed(2).replace(".", ",")} €`;
}

// Mock data for the frontend prototype. No backend/API yet — see ../../VISION.md.

export interface Merchant {
  id: string;
  name: string;
  category: string;
  distance: string;
  rating: number;
  followed: boolean;
}

export interface Offer {
  id: string;
  merchantId: string;
  title: string;
  priceOriginal: number;
  pricePromo: number;
  pickupWindow: string;
  quantityLeft: number;
  status: "active" | "exhausted";
}

export const merchants: Merchant[] = [
  {
    id: "boulangerie-martin",
    name: "Boulangerie Martin",
    category: "Boulangerie",
    distance: "350 m",
    rating: 4.8,
    followed: true,
  },
  {
    id: "primeur-des-halles",
    name: "Primeur des Halles",
    category: "Primeur",
    distance: "520 m",
    rating: 4.6,
    followed: false,
  },
];

export const offers: Offer[] = [
  {
    id: "panier-boulanger-surprise",
    merchantId: "boulangerie-martin",
    title: "Panier boulanger surprise",
    priceOriginal: 8.0,
    pricePromo: 3.2,
    pickupWindow: "18h30 – 19h30",
    quantityLeft: 4,
    status: "active",
  },
  {
    id: "panier-viennoiseries",
    merchantId: "boulangerie-martin",
    title: "Panier viennoiseries",
    priceOriginal: 7.0,
    pricePromo: 2.8,
    pickupWindow: "19h00 – 19h30",
    quantityLeft: 5,
    status: "active",
  },
  {
    id: "panier-pain-du-jour",
    merchantId: "boulangerie-martin",
    title: "Panier pain du jour",
    priceOriginal: 6.0,
    pricePromo: 3.2,
    pickupWindow: "18h00 – 18h30",
    quantityLeft: 0,
    status: "exhausted",
  },
  {
    id: "panier-de-fruits",
    merchantId: "primeur-des-halles",
    title: "Panier de fruits",
    priceOriginal: 8.9,
    pricePromo: 3.5,
    pickupWindow: "17h00 – 18h00",
    quantityLeft: 3,
    status: "active",
  },
];

export function getMerchant(id: string): Merchant | undefined {
  return merchants.find((m) => m.id === id);
}

export function getOffer(id: string): Offer | undefined {
  return offers.find((o) => o.id === id);
}

export function getMerchantOffers(merchantId: string): Offer[] {
  return offers.filter((o) => o.merchantId === merchantId);
}

export function discountPercent(offer: Offer): number {
  return Math.round((1 - offer.pricePromo / offer.priceOriginal) * 100);
}

export function reservationCodeFor(offer: Offer): string {
  const hash = Array.from(offer.id).reduce((acc, ch) => acc + ch.charCodeAt(0), 0);
  return `DC-${4000 + (hash % 999)}`;
}

export function formatPrice(value: number): string {
  return `${value.toFixed(2).replace(".", ",")} €`;
}

// Session tokens: marchand et consommateur sont deux principaux distincts
// côté backend (JWT séparés), donc deux clés séparées ici aussi.

const CONSUMER_KEY = "dc_consumer_token";
const CONSUMER_EMAIL_KEY = "dc_consumer_email";
const MERCHANT_KEY = "dc_merchant_token";
const ADMIN_KEY = "dc_admin_token";

function read(key: string): string | null {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(key);
}

function write(key: string, value: string) {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(key, value);
}

function clear(key: string) {
  if (typeof localStorage === "undefined") return;
  localStorage.removeItem(key);
}

export const getConsumerToken = () => read(CONSUMER_KEY);
export const setConsumerToken = (token: string) => write(CONSUMER_KEY, token);
export const clearConsumerToken = () => clear(CONSUMER_KEY);

export const getConsumerEmail = () => read(CONSUMER_EMAIL_KEY);
export const setConsumerEmail = (email: string) => write(CONSUMER_EMAIL_KEY, email);

export const getMerchantToken = () => read(MERCHANT_KEY);
export const setMerchantToken = (token: string) => write(MERCHANT_KEY, token);
export const clearMerchantToken = () => clear(MERCHANT_KEY);

export const getAdminToken = () => read(ADMIN_KEY);
export const setAdminToken = (token: string) => write(ADMIN_KEY, token);
export const clearAdminToken = () => clear(ADMIN_KEY);

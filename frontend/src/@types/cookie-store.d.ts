interface CookieStoreSetOptions {
  name: string;
  value: string;
  expires?: number | Date;
  path?: string;
  domain?: string;
  secure?: boolean;
  sameSite?: "strict" | "lax" | "none";
}

interface CookieStore {
  set(options: CookieStoreSetOptions): Promise<void>;
}

declare var cookieStore: CookieStore;

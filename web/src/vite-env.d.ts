/// <reference types="vite/client" />
interface ImportMetaEnv {
  /** Absolute API origin for a wrapped app, e.g. https://api.lagn.app. Empty: same origin. */
  readonly VITE_API_BASE?: string;
}
interface ImportMeta { readonly env: ImportMetaEnv }

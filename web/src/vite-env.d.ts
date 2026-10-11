/// <reference types="vite/client" />
interface ImportMetaEnv {
  /** Absolute API origin for a wrapped app, e.g. https://api.lagn.app. Empty: same origin. */
  readonly VITE_API_BASE?: string;
  /** The professional-surface kill switch. Unset or anything but
   *  off/0/false/no leaves it on. Read only as a fallback under the test
   *  runner; a real build folds PRO_BUILD instead. */
  readonly VITE_PRO?: string;
}
interface ImportMeta { readonly env: ImportMetaEnv }

/**
 * Build-time literals the bundler substitutes.
 *
 * `PRO_BUILD` is the owner's professional-surface kill switch. It is a literal
 * rather than a lookup so the bundler can delete the branch that imports the
 * professional chunk, which is what makes a surface-off build genuinely omit
 * the code instead of shipping something unreachable.
 */
declare const PRO_BUILD: boolean;
declare const __ENGINE_VERSION__: string;

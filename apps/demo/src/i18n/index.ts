import { en } from './en';
import type { TranslationKey } from './en';

export type Locale = 'en';

const locales = { en } satisfies Record<Locale, typeof en>;

/** Resolve translations for a given locale, falling back to English. */
export function getTranslations(locale: Locale = 'en') {
  return locales[locale] ?? locales.en;
}

/** Build a t() helper bound to a locale. */
export function createTranslator(locale: Locale = 'en') {
  const dict = getTranslations(locale);
  return (key: TranslationKey): string => dict[key];
}

export type { TranslationKey };

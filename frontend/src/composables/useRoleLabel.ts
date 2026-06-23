import type { ComposerTranslation } from "vue-i18n";

const ROLE_KEYS = ["admin", "operator", "readonly", "viewer", "guest"] as const;

export const roleLabelOf = (t: ComposerTranslation, role: string) => {
  const key = ROLE_KEYS.includes(role as (typeof ROLE_KEYS)[number]) ? role : "guest";
  return t(`roles.${key}`);
};

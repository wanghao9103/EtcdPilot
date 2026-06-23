import type { ComposerTranslation } from "vue-i18n";

export const buildGreeting = (t: ComposerTranslation, username: string) => {
  const hour = new Date().getHours();
  let key = "greeting.hello";
  if (hour < 6) key = "greeting.lateNight";
  else if (hour < 12) key = "greeting.morning";
  else if (hour < 14) key = "greeting.noon";
  else if (hour < 18) key = "greeting.afternoon";
  else key = "greeting.evening";

  const period = t(key);
  return username ? t("greeting.withName", { period, name: username }) : period;
};

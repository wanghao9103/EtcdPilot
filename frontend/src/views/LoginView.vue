<template>
  <section class="auth-wrap">
    <div class="auth-glow" aria-hidden="true"></div>
    <section class="panel auth-panel">
      <div class="auth-header">
        <BrandIcon size="lg" />
        <div class="auth-titles">
          <h2>{{ t("login.title") }}</h2>
          <p class="hint">{{ t("login.subtitle") }}</p>
        </div>
      </div>
      <form @submit.prevent="submit">
        <label>
          {{ t("login.username") }}
          <input v-model="form.username" :placeholder="t('login.usernamePh')" autocomplete="username" required />
        </label>
        <label>
          {{ t("login.password") }}
          <input
            type="password"
            v-model="form.password"
            :placeholder="t('login.passwordPh')"
            autocomplete="current-password"
            required
          />
        </label>
        <label class="remember-row">
          <input v-model="rememberMe" type="checkbox" class="remember-check" />
          <span>{{ t("login.remember") }}</span>
        </label>
        <button type="submit" class="primary" :disabled="submitting">
          <span v-if="submitting" class="spinner"></span>
          {{ submitting ? t("login.submitting") : t("login.submit") }}
        </button>
      </form>
      <p v-if="error" class="message error">{{ error }}</p>
    </section>
  </section>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { useAuthStore } from "../stores/auth";
import BrandIcon from "../components/BrandIcon.vue";
import { loadRememberedLogin, saveRememberedLogin } from "../utils/rememberLogin";

const { t } = useI18n();
const router = useRouter();
const auth = useAuthStore();
const error = ref("");
const submitting = ref(false);
const rememberMe = ref(false);
const form = reactive({ username: "", password: "" });

onMounted(() => {
  const saved = loadRememberedLogin();
  if (!saved?.remember) return;
  rememberMe.value = true;
  form.username = saved.username;
  form.password = saved.password;
});

const submit = async () => {
  error.value = "";
  submitting.value = true;
  try {
    await auth.login(form);
    saveRememberedLogin({
      remember: rememberMe.value,
      username: form.username,
      password: form.password,
    });
    router.push("/");
  } catch (e: any) {
    error.value = e.message || t("login.failed");
  } finally {
    submitting.value = false;
  }
};
</script>

<style scoped>
.auth-wrap {
  position: relative;
  display: flex;
  justify-content: center;
  width: 100%;
  padding: 24px;
}

.auth-glow {
  position: absolute;
  top: 10%;
  width: min(480px, 90vw);
  height: min(480px, 90vw);
  border-radius: 50%;
  background: radial-gradient(circle, rgba(34, 211, 238, 0.14), transparent 68%);
  pointer-events: none;
  filter: blur(2px);
}

.auth-panel {
  position: relative;
  width: 100%;
  max-width: 420px;
  padding: 32px 28px;
  border-color: var(--border-strong);
  box-shadow:
    var(--shadow),
    0 0 60px rgba(34, 211, 238, 0.06);
}

.auth-panel::before {
  left: 18%;
  right: 18%;
}

.auth-header {
  text-align: center;
  margin-bottom: 28px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.auth-titles h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.auth-titles .hint {
  margin: 8px 0 0;
  font-size: 13px;
  line-height: 1.5;
}

form {
  display: grid;
  gap: 18px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 7px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.remember-row {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  margin-top: -4px;
  cursor: pointer;
  user-select: none;
}

.remember-check {
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: var(--primary);
  cursor: pointer;
}

button.primary {
  width: 100%;
  padding: 12px;
  margin-top: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
}

.message.error {
  margin-top: 16px;
  margin-bottom: 0;
}
</style>

import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { createRouter, createWebHashHistory, type RouteLocationNormalized, type NavigationGuardNext } from "vue-router";

import LoginView from "./views/LoginView.vue";
import DashboardView from "./views/DashboardView.vue";
import ClustersView from "./views/ClustersView.vue";
import KeysView from "./views/KeysView.vue";
import AuditsView from "./views/AuditsView.vue";
import LeasesView from "./views/LeasesView.vue";
import ServicesView from "./views/ServicesView.vue";
import { i18n } from "./i18n";
import { useAuthStore } from "./stores/auth";

const routes = [
  { path: "/", component: DashboardView },
  { path: "/login", component: LoginView },
  { path: "/clusters", component: ClustersView },
  { path: "/keys", component: KeysView },
  { path: "/audits", component: AuditsView },
  { path: "/leases", component: LeasesView },
  { path: "/services", component: ServicesView },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const app = createApp(App);
app.use(createPinia());
app.use(i18n);
app.use(router);

router.beforeEach(async (_to: RouteLocationNormalized, _from: RouteLocationNormalized, next: NavigationGuardNext) => {
  const auth = useAuthStore();
  if (_to.path === "/login") {
    next();
    return;
  }
  try {
    if (!auth.loggedIn) {
      await auth.fetchMe();
    }
    if (auth.loggedIn) {
      next();
      return;
    }
  } catch {
    next("/login");
    return;
  }
  next("/login");
});

app.mount("#app");

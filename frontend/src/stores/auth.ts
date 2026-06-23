import { defineStore } from "pinia";
import { ref } from "vue";
import api from "../api";

export const useAuthStore = defineStore("auth", () => {
  const username = ref("");
  const role = ref("");
  const loggedIn = ref(false);
  const permissions = ref<string[]>([]);

  const login = async (payload: { username: string; password: string }) => {
    const resp = await api.post("/auth/login", payload);
    username.value = resp.data.username;
    role.value = resp.data.role;
    permissions.value = resp.data.permissions || [];
    loggedIn.value = true;
  };

  const fetchMe = async () => {
    const resp = await api.get("/me");
    username.value = resp.data.username;
    role.value = resp.data.role;
    permissions.value = resp.data.permissions || [];
    loggedIn.value = true;
  };

  const logout = async () => {
    await api.post("/auth/logout");
    username.value = "";
    role.value = "";
    permissions.value = [];
    loggedIn.value = false;
  };

  return { username, role, loggedIn, permissions, login, fetchMe, logout };
});

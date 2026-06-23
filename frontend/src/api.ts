import axios from "axios";

export const api = axios.create({
  baseURL: "/api",
  timeout: 10000,
  headers: {
    "Content-Type": "application/json",
  },
});

api.interceptors.response.use(
  (res) => res,
  (error) => {
    const message = error?.response?.data?.message || "请求失败";
    return Promise.reject(new Error(message));
  },
);

export default api;

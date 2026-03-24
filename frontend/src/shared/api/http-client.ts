import axios, { type AxiosError } from 'axios';

import { useAuthStore } from '@/stores/authStore';
import type { ApiError } from '@/types';

export const httpClient = axios.create({
  baseURL: '/api',
  timeout: 15_000,
  headers: {
    'Content-Type': 'application/json',
  },
});

httpClient.interceptors.request.use(
  (config) => {
    const token = useAuthStore.getState().token;
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error),
);

httpClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiError>) => {
    if (error.response?.status === 401) {
      useAuthStore.getState().logout();
    }
    return Promise.reject(error);
  },
);

export default httpClient;

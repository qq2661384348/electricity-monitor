import axios, { type AxiosError, type InternalAxiosRequestConfig } from 'axios';

import { useAuthStore } from '@/stores/authStore';
import type { ApiError, LoginResponse } from '@/types';

declare module 'axios' {
  interface InternalAxiosRequestConfig {
    _retry?: boolean;
    _skipAuthRefresh?: boolean;
  }
}

export const httpClient = axios.create({
  baseURL: '/api',
  timeout: 15_000,
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

let refreshInFlight: Promise<string | null> | null = null;

const isAuthEndpoint = (config: InternalAxiosRequestConfig) =>
  typeof config.url === 'string' && config.url.startsWith('/auth/');

const refreshAccessToken = async (): Promise<string | null> => {
  if (!refreshInFlight) {
    refreshInFlight = httpClient
      .post<LoginResponse>(
        '/auth/refresh',
        undefined,
        { _skipAuthRefresh: true } as InternalAxiosRequestConfig,
      )
      .then((response) => {
        const data = response.data as LoginResponse;
        useAuthStore.getState().login(data.access_token, data.user);
        return data.access_token;
      })
      .catch((error) => {
        useAuthStore.getState().logout();
        throw error;
      })
      .finally(() => {
        refreshInFlight = null;
      });
  }

  return refreshInFlight;
};

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
  async (error: AxiosError<ApiError>) => {
    const originalRequest = error.config;
    if (!originalRequest || error.response?.status !== 401) {
      return Promise.reject(error);
    }

    if (originalRequest._skipAuthRefresh || isAuthEndpoint(originalRequest)) {
      if (originalRequest.url === '/auth/refresh' || originalRequest.url === '/auth/logout') {
        useAuthStore.getState().logout();
      }
      return Promise.reject(error);
    }

    if (originalRequest._retry) {
      useAuthStore.getState().logout();
      return Promise.reject(error);
    }

    originalRequest._retry = true;

    try {
      const refreshedAccessToken = await refreshAccessToken();
      if (!refreshedAccessToken) {
        return Promise.reject(error);
      }

      originalRequest.headers.Authorization = `Bearer ${refreshedAccessToken}`;
      return httpClient(originalRequest);
    } catch (refreshError) {
      return Promise.reject(refreshError);
    }
  },
);

export default httpClient;

import { create } from 'zustand';
import type { User } from '@/types';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isSessionReady: boolean;
  
  // Actions
  login: (token: string, user: User) => void;
  logout: () => void;
  updateUser: (user: User) => void;
  markSessionReady: () => void;
}

export const useAuthStore = create<AuthState>()((set) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isSessionReady: false,

  login: (token: string, user: User) => {
    set({ token, user, isAuthenticated: true, isSessionReady: true });
  },

  logout: () => {
    set({ token: null, user: null, isAuthenticated: false, isSessionReady: true });
  },

  updateUser: (user: User) => {
    set({ user });
  },

  markSessionReady: () => {
    set((state) => ({ ...state, isSessionReady: true }));
  },
}));

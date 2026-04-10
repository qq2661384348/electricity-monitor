import { useEffect } from 'react';

import { authApi } from '@/features/auth-login';
import { useAuthStore } from '@/stores/authStore';

import { AppRouter } from './routes';

let sessionBootstrapInFlight: Promise<void> | null = null;

function App() {
  const isSessionReady = useAuthStore((state) => state.isSessionReady);
  const login = useAuthStore((state) => state.login);
  const logout = useAuthStore((state) => state.logout);

  useEffect(() => {
    if (isSessionReady) {
      return;
    }

    if (!sessionBootstrapInFlight) {
      sessionBootstrapInFlight = authApi
        .refreshSession()
        .then((response) => {
          login(response.access_token, response.user);
        })
        .catch(() => {
          logout();
        })
        .finally(() => {
          sessionBootstrapInFlight = null;
        });
    }
  }, [isSessionReady, login, logout]);

  if (!isSessionReady) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-linear-to-br from-yellow-100 via-orange-100 to-pink-100">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-orange-400 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-orange-700 font-semibold">正在恢复登录状态...</p>
        </div>
      </div>
    );
  }

  return <AppRouter />;
}

export default App;

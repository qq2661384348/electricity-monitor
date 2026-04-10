import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { useAuthStore } from '@/stores/authStore';

import App from './App';

const { refreshSessionMock } = vi.hoisted(() => ({
  refreshSessionMock: vi.fn(),
}));

vi.mock('@/features/auth-login', () => ({
  authApi: {
    refreshSession: refreshSessionMock,
  },
}));

vi.mock('./routes', () => ({
  AppRouter: () => <div>router-ready</div>,
}));

describe('App', () => {
  beforeEach(() => {
    refreshSessionMock.mockReset();
    useAuthStore.setState({
      user: null,
      token: null,
      isAuthenticated: false,
      isSessionReady: false,
    });
  });

  it('restores session before rendering router', async () => {
    refreshSessionMock.mockResolvedValue({
      access_token: 'bootstrap-token',
      token_type: 'Bearer',
      expires_in: 3600,
      user: {
        id: 'user-1',
        qq_number: '123456789',
        role: 'user',
        is_active: true,
      },
    });

    render(<App />);

    expect(screen.getByText('正在恢复登录状态...')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText('router-ready')).toBeInTheDocument();
    });

    expect(useAuthStore.getState().isAuthenticated).toBe(true);
    expect(useAuthStore.getState().token).toBe('bootstrap-token');
    expect(refreshSessionMock).toHaveBeenCalledTimes(1);
  });
});

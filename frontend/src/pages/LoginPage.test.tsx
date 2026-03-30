import { http, HttpResponse } from 'msw';
import userEvent from '@testing-library/user-event';
import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { server } from '@/test/msw/server';
import { renderWithProviders } from '@/test/render';
import { useAuthStore } from '@/stores/authStore';

import LoginPage from './LoginPage';

describe('LoginPage', () => {
  it('writes token and user state after successful login', async () => {
    const user = userEvent.setup();
    renderWithProviders(<LoginPage />, { route: '/login' });

    await user.type(screen.getByLabelText('QQ号码'), '123456789');
    await user.type(screen.getByLabelText('验证码'), '123456');
    await user.click(screen.getByRole('button', { name: '确认进入' }));

    await waitFor(() => {
      expect(useAuthStore.getState().isAuthenticated).toBe(true);
    });

    expect(useAuthStore.getState().token).toBe('access-token');
    expect(useAuthStore.getState().user?.qq_number).toBe('123456789');
  });

  it('shows backend error detail when login fails', async () => {
    server.use(
      http.post('*/api/auth/verify-and-login', () =>
        HttpResponse.json({ detail: '验证码已失效' }, { status: 401 }),
      ),
    );

    const user = userEvent.setup();
    renderWithProviders(<LoginPage />, { route: '/login' });

    await user.type(screen.getByLabelText('QQ号码'), '123456789');
    await user.type(screen.getByLabelText('验证码'), '123456');
    await user.click(screen.getByRole('button', { name: '确认进入' }));

    expect(await screen.findByText('验证码已失效')).toBeInTheDocument();
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
  });
});

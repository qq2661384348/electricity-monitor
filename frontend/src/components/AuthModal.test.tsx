import { http, HttpResponse } from 'msw';
import userEvent from '@testing-library/user-event';
import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { server } from '@/test/msw/server';
import { renderWithProviders } from '@/test/render';
import { useAuthStore } from '@/stores/authStore';

import { AuthModal } from './AuthModal';

describe('AuthModal', () => {
  it('shows backend error detail when login fails', async () => {
    server.use(
      http.post('*/api/auth/verify-and-login', () =>
        HttpResponse.json({ detail: '验证码已失效' }, { status: 401 }),
      ),
    );

    const user = userEvent.setup();
    useAuthStore.setState({ isSessionReady: true });
    renderWithProviders(<AuthModal isOpen onClose={vi.fn()} />);

    await user.type(screen.getByLabelText('QQ号码'), '123456789');
    await user.type(screen.getByLabelText('验证码'), '123456');
    await user.click(screen.getByRole('button', { name: '确认进入' }));

    expect(await screen.findByText('验证码已失效')).toBeInTheDocument();
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
  });
});

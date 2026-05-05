import userEvent from '@testing-library/user-event';
import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useAuthStore } from '@/stores/authStore';
import { renderWithProviders } from '@/test/render';

import { Navbar } from './Navbar';

describe('Navbar', () => {
  it('allows guests to view the announcement', async () => {
    const user = userEvent.setup();
    useAuthStore.setState({
      token: null,
      user: null,
      isAuthenticated: false,
      isSessionReady: true,
    });

    renderWithProviders(<Navbar onLoginClick={vi.fn()} />);

    await user.click(screen.getByRole('button', { name: '查看公告' }));

    expect(screen.getByText('项目源码已公开发布')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'https://github.com/qq2661384348/electricity-monitor' })).toBeInTheDocument();
    expect(useAuthStore.getState().isAuthenticated).toBe(false);
  });

  it('requires confirmation before logout', async () => {
    const user = userEvent.setup();
    useAuthStore.setState({
      token: 'access-token',
      user: {
        id: 'user-1',
        qq_number: '123456789',
        role: 'user',
        is_active: true,
      },
      isAuthenticated: true,
      isSessionReady: true,
    });

    renderWithProviders(<Navbar onLoginClick={vi.fn()} />);

    await user.click(screen.getByRole('button', { name: '退出登录' }));

    expect(screen.getByText('是否确认退出登录？')).toBeInTheDocument();
    expect(useAuthStore.getState().isAuthenticated).toBe(true);

    await user.click(screen.getByRole('button', { name: '取消' }));

    await waitFor(() => {
      expect(screen.queryByText('是否确认退出登录？')).not.toBeInTheDocument();
    });
    expect(useAuthStore.getState().isAuthenticated).toBe(true);

    await user.click(screen.getByRole('button', { name: '退出登录' }));
    await user.click(screen.getByRole('button', { name: '确认退出' }));

    await waitFor(() => {
      expect(useAuthStore.getState().isAuthenticated).toBe(false);
    });
  });
});

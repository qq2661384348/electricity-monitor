import { act, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { bindingKeys } from '@/shared/api/queryKeys';
import { renderHookWithProviders } from '@/test/render';
import type { User } from '@/types';
import { useAuthStore } from '@/stores/authStore';

import { useDashboardPage } from './useDashboardPage';

const authenticatedUser: User = {
  id: 'user-1',
  qq_number: '123456789',
  role: 'user',
  is_active: true,
};

describe('useDashboardPage', () => {
  it('opens auth modal for unauthenticated users before room detail', async () => {
    const { result } = renderHookWithProviders(() => useDashboardPage());

    const room = {
      id: 'room-1',
      roomid: 1001,
      electricity_fee: 23.5,
      send_flag: false,
      threshold: 30,
      room_name: '101',
      primary_roompath: '校区A/宿舍楼A/1楼/101',
      has_additional_paths: false,
      is_active: true,
      source_type: 'crawler',
      created_at: '2026-03-25T00:00:00Z',
      updated_at: '2026-03-25T00:00:00Z',
      bindingId: 'binding-1',
    };

    act(() => {
      result.current.handleRoomClick(room);
    });

    expect(result.current.isAuthModalOpen).toBe(true);
    expect(result.current.isDetailModalOpen).toBe(false);
  });

  it('hydrates rooms from bindings and invalidates queries after mutation', async () => {
    useAuthStore.setState({
      user: authenticatedUser,
      token: 'access-token',
      isAuthenticated: true,
      isSessionReady: true,
    });

    const { queryClient, result } = renderHookWithProviders(() => useDashboardPage());
    const invalidateQueriesSpy = vi.spyOn(queryClient, 'invalidateQueries');

    await waitFor(() => {
      expect(result.current.rooms).toHaveLength(1);
    });

    act(() => {
      result.current.handleRoomClick(result.current.rooms[0]!);
    });

    expect(result.current.isDetailModalOpen).toBe(true);
    expect(result.current.selectedRoom?.bindingId).toBe('binding-1');

    await act(async () => {
      await result.current.handleToggleNotification('binding-1', false);
    });

    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: bindingKeys.all });
  });
});

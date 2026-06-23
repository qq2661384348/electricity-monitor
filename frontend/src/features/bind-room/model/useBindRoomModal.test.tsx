import { act, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { bindingKeys, roomKeys } from '@/shared/api/queryKeys';
import { useAuthStore } from '@/stores/authStore';
import { renderHookWithProviders } from '@/test/render';
import { useBindRoomModal } from './useBindRoomModal';

type BindRoomModalState = ReturnType<typeof useBindRoomModal>;

async function selectDefaultRoom(result: { current: BindRoomModalState }) {
  await waitFor(() => {
    expect(result.current.options).toHaveLength(1);
  });

  await act(async () => {
    await result.current.handleSelectOption({
      name: '校区A',
      is_leaf: false,
      room_count: 1,
    });
  });

  await waitFor(() => {
    expect(result.current.currentStep).toBe(2);
    expect(result.current.options[0]?.name).toBe('101');
  });

  await act(async () => {
    await result.current.handleSelectOption({
      name: '101',
      is_leaf: true,
      room_count: 1,
      roomid: '1001',
    });
  });

  await waitFor(() => {
    expect(result.current.finalRoom?.roomid).toBe('1001');
    expect(result.current.currentStep).toBe(5);
  });
}

describe('useBindRoomModal', () => {
  it('binds a room and invalidates related queries', async () => {
    const onClose = vi.fn();
    const onSuccess = vi.fn();
    const { queryClient, result } = renderHookWithProviders(() =>
      useBindRoomModal({
        isOpen: true,
        onClose,
        onSuccess,
      }),
    );
    const invalidateQueriesSpy = vi.spyOn(queryClient, 'invalidateQueries');

    await selectDefaultRoom(result);

    await act(async () => {
      await result.current.handleBind();
    });

    expect(onSuccess).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: roomKeys.all });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: roomKeys.flagged() });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: bindingKeys.all });
  });

  it('shows an error when a leaf node is missing roomid', async () => {
    const { result } = renderHookWithProviders(() =>
      useBindRoomModal({
        isOpen: true,
        onClose: vi.fn(),
      }),
    );

    await waitFor(() => {
      expect(result.current.options).toHaveLength(1);
    });

    await act(async () => {
      await result.current.handleSelectOption({
        name: '校区A',
        is_leaf: false,
        room_count: 1,
      });
    });

    await waitFor(() => {
      expect(result.current.options[0]?.name).toBe('101');
    });

    await act(async () => {
      await result.current.handleSelectOption({
        name: '101',
        is_leaf: true,
        room_count: 1,
      });
    });

    await waitFor(() => {
      expect(result.current.error).toBe('查询房间失败，请稍后重试');
    });
  });

  it('allows admin users to bind a room', async () => {
    useAuthStore.setState({
      user: {
        id: 'admin-1',
        login_mode: 'qq',
        identifier: '2661384348',
        qq_number: '2661384348',
        email: null,
        role: 'admin',
        is_active: true,
      },
      token: 'admin-token',
      isAuthenticated: true,
      isSessionReady: true,
    });

    const onClose = vi.fn();
    const onSuccess = vi.fn();
    const { result } = renderHookWithProviders(() =>
      useBindRoomModal({
        isOpen: true,
        onClose,
        onSuccess,
      }),
    );

    await selectDefaultRoom(result);

    await act(async () => {
      await result.current.handleBind();
    });

    expect(result.current.error).toBeNull();
    expect(onSuccess).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
  });
});

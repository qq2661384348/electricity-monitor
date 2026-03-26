import { act, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { describe, expect, it, vi } from 'vitest';

import { bindingKeys, roomKeys } from '@/shared/api/queryKeys';
import { renderHookWithProviders } from '@/test/render';
import { server } from '@/test/msw/server';

import { useBindRoomModal } from './useBindRoomModal';

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
      });
    });

    await waitFor(() => {
      expect(result.current.finalRoom?.roomid).toBe(1001);
      expect(result.current.currentStep).toBe(5);
    });

    await act(async () => {
      await result.current.handleBind();
    });

    expect(onSuccess).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: roomKeys.all });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: roomKeys.flagged() });
    expect(invalidateQueriesSpy).toHaveBeenCalledWith({ queryKey: bindingKeys.all });
  });

  it('shows an error when room lookup fails', async () => {
    server.use(
      http.get('*/api/rooms/by-path', () =>
        HttpResponse.json({ message: 'lookup failed' }, { status: 500 }),
      ),
    );

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
});

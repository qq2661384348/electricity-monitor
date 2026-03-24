import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';

import { bindingApi } from '@/entities/binding';
import { roomApi } from '@/entities/room';
import { bindingKeys } from '@/shared/api/queryKeys';
import { useAuthStore } from '@/stores/authStore';
import type { Binding, Room } from '@/types';

import { useBindingsQuery } from '../hooks/useDashboardData';

type BoundRoom = Room & { bindingId?: string };

export function useDashboardPage() {
  const [isAuthModalOpen, setIsAuthModalOpen] = useState(false);
  const [isBindModalOpen, setIsBindModalOpen] = useState(false);
  const [isThresholdModalOpen, setIsThresholdModalOpen] = useState(false);
  const [selectedRoom, setSelectedRoom] = useState<BoundRoom | null>(null);
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false);

  const { isAuthenticated } = useAuthStore();
  const queryClient = useQueryClient();
  const { data: bindings = [], isLoading: isLoadingBindings } = useBindingsQuery();

  const rooms: BoundRoom[] = bindings
    .filter((binding) => binding.room)
    .map((binding) => ({
      ...binding.room!,
      bindingId: binding.id,
    }));

  const requireAuth = (action: () => void) => {
    if (isAuthenticated) {
      action();
      return;
    }

    setIsAuthModalOpen(true);
  };

  const handleRoomClick = (room: BoundRoom) => {
    requireAuth(() => {
      setSelectedRoom(room);
      setIsDetailModalOpen(true);
    });
  };

  const handleEditThreshold = (room: Room) => {
    setSelectedRoom(room);
    setIsThresholdModalOpen(true);
  };

  const handleConfirmThreshold = async (value: string) => {
    if (!selectedRoom) return;

    await roomApi.updateThreshold(selectedRoom.id, Number(value));
    await queryClient.invalidateQueries({ queryKey: bindingKeys.all });
  };

  const handleToggleNotification = async (bindingId: string, enabled: boolean) => {
    await bindingApi.updateNotificationEnabled(bindingId, enabled);
    await queryClient.invalidateQueries({ queryKey: bindingKeys.all });
  };

  const handleDeleteBinding = async (bindingId: string) => {
    await bindingApi.deleteBinding(bindingId);
    await queryClient.invalidateQueries({ queryKey: bindingKeys.all });
  };

  const getBindingForRoom = (room: BoundRoom): Binding | undefined =>
    room.bindingId ? bindings.find((binding) => binding.id === room.bindingId) : undefined;

  return {
    bindings,
    getBindingForRoom,
    handleConfirmThreshold,
    handleDeleteBinding,
    handleEditThreshold,
    handleRoomClick,
    handleToggleNotification,
    isAuthModalOpen,
    isAuthenticated,
    isBindModalOpen,
    isDetailModalOpen,
    isLoading: isLoadingBindings,
    isThresholdModalOpen,
    openAuthModal: () => setIsAuthModalOpen(true),
    openBindModal: () => setIsBindModalOpen(true),
    rooms,
    selectedRoom,
    setIsAuthModalOpen,
    setIsBindModalOpen,
    setIsDetailModalOpen,
    setIsThresholdModalOpen,
    setSelectedRoom,
  };
}

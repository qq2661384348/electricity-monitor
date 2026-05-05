import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';

import { usePathTree } from '@/hooks/usePathTree';
import { bindingKeys, roomKeys } from '@/shared/api/queryKeys';
import type { PathChild } from '@/types';

import { bindRoomApi } from '../api/bindRoomApi';

interface SelectedRoomPreview {
  readonly roomid: number;
  readonly room_name: string;
  readonly primary_roompath: string;
}

interface UseBindRoomModalOptions {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess?: () => void;
}

export function useBindRoomModal({
  isOpen,
  onClose,
  onSuccess,
}: UseBindRoomModalOptions) {
  const [currentStep, setCurrentStep] = useState(1);
  const [selectedPath, setSelectedPath] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [finalRoom, setFinalRoom] = useState<SelectedRoomPreview | null>(null);
  const [isTouchDevice, setIsTouchDevice] = useState(false);

  const queryClient = useQueryClient();
  const stepLabels = ['校区', '建筑', '楼层', '房间'];
  const parent = selectedPath.join('/');
  const {
    data: pathData,
    isLoading,
    error: queryError,
    refetch,
  } = usePathTree(parent, isOpen && currentStep < 5);

  const options = pathData?.children ?? [];

  useEffect(() => {
    if (typeof globalThis !== 'undefined' && globalThis.window) {
      const mediaQuery = globalThis.window.matchMedia('(pointer: coarse), (hover: none)');
      const updateMatch = () => setIsTouchDevice(mediaQuery.matches);
      updateMatch();
      mediaQuery.addEventListener?.('change', updateMatch);
      return () => mediaQuery.removeEventListener?.('change', updateMatch);
    }
    return undefined;
  }, []);

  const reset = () => {
    setCurrentStep(1);
    setSelectedPath([]);
    setError(null);
    setFinalRoom(null);
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  const handleSelectOption = (option: PathChild) => {
    const newPath = [...selectedPath, option.name];
    setSelectedPath(newPath);
    setError(null);

    if (option.is_leaf) {
      if (typeof option.roomid !== 'number') {
        setError('查询房间失败，请稍后重试');
        return;
      }

      setFinalRoom({
        roomid: option.roomid,
        room_name: option.name,
        primary_roompath: newPath.join('/'),
      });
      setCurrentStep(5);
      return;
    }

    setCurrentStep((step) => step + 1);
  };

  const handleGoBack = () => {
    if (currentStep > 1) {
      setSelectedPath((current) => current.slice(0, -1));
      setCurrentStep((step) => step - 1);
      setError(null);
    }
  };

  const handleRestart = () => {
    reset();
  };

  const clearError = () => {
    setError(null);
  };

  const handleBind = async () => {
    if (!finalRoom) return;

    setError(null);
    try {
      await bindRoomApi.createBinding(finalRoom.roomid);
      await queryClient.invalidateQueries({ queryKey: roomKeys.all });
      await queryClient.invalidateQueries({ queryKey: roomKeys.flagged() });
      await queryClient.invalidateQueries({ queryKey: bindingKeys.all });

      onSuccess?.();
      handleClose();
    } catch {
      setError('绑定失败，请稍后重试或联系管理员');
    }
  };

  return {
    currentStep,
    clearError,
    error,
    finalRoom,
    handleBind,
    handleClose,
    handleGoBack,
    handleRestart,
    handleSelectOption,
    isLoading,
    isTouchDevice,
    options,
    queryError,
    refetch,
    selectedPath,
    stepLabels,
  };
}

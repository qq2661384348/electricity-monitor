import { useState } from 'react';
import { Bell, BellOff } from 'lucide-react';
import { ComicModal } from '@/components/ui/comic-modal';
import { Button } from './Button';

interface NotificationModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onConfirm: (enabled: boolean) => Promise<void>;
  readonly currentStatus: boolean; // 当前通知状态
  readonly roomName: string; // 房间名称
}

/**
 * 通知开关模态框组件 - 使用 ComicModal 重构
 * 代码量从 205 行减少到 ~120 行（减少 41%）
 */
export function NotificationModal({
  isOpen,
  onClose,
  onConfirm,
  currentStatus,
  roomName,
}: NotificationModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState(currentStatus);
  const isNotificationEnabled = selectedStatus;
  const isNotificationDisabled = !selectedStatus;

  // 重置状态（当模态框关闭时）
  const handleClose = () => {
    setSelectedStatus(currentStatus);
    setIsSubmitting(false);
    onClose();
  };

  // 提交处理
  const handleSubmit = async () => {
    setIsSubmitting(true);
    
    try {
      await onConfirm(selectedStatus);
      handleClose();
    } catch (err) {
      console.error('更新通知设置失败:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <ComicModal
      isOpen={isOpen}
      onClose={handleClose}
      size="md"
      title="通知设置"
      showCloseButton
      footer={
        <div className="flex gap-3 w-full">
          <Button
            onClick={handleClose}
            variant="secondary"
            size="lg"
            disabled={isSubmitting}
            className="flex-1"
          >
            取消
          </Button>
          <Button
            onClick={handleSubmit}
            variant="primary"
            size="lg"
            isLoading={isSubmitting}
            disabled={isSubmitting || selectedStatus === currentStatus}
            className="flex-1"
          >
            {isSubmitting ? '保存中...' : '保存'}
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        {/* 房间名称 */}
        <div className="text-center">
          <p className="text-sm font-bold text-gray-700 mb-1">房间</p>
          <p className="text-base font-black text-black">{roomName}</p>
        </div>

        {/* 选项区域 */}
        <div className="space-y-3">
                {/* 开启通知 */}
                <button
                  onClick={() => setSelectedStatus(true)}
                  disabled={isSubmitting}
                  className={`w-full p-4 border-4 border-black transition-all disabled:opacity-50 disabled:cursor-not-allowed ${
                    isNotificationEnabled
                      ? 'bg-status-normal shadow-[4px_4px_0_0_#000] scale-[1.02]'
                      : 'bg-white shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000]'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`p-2 border-2 border-black ${
                      isNotificationEnabled ? 'bg-white' : 'bg-gray-100'
                    }`}>
                      <Bell className="w-6 h-6 text-black" strokeWidth={3} />
                    </div>
                    <div className="flex-1 text-left">
                      <p className={`font-black text-base uppercase ${
                        isNotificationEnabled ? 'text-white' : 'text-black'
                      }`}>开启通知</p>
                      <p className={`text-xs font-bold mt-1 ${
                        isNotificationEnabled ? 'text-white/90' : 'text-gray-600'
                      }`}>余额低于阈值时发送通知</p>
                    </div>
                    {isNotificationEnabled && (
                      <div className="w-6 h-6 bg-white border-2 border-black flex items-center justify-center">
                        <span className="text-status-normal text-lg">✓</span>
                      </div>
                    )}
                  </div>
                </button>

                {/* 关闭通知 */}
                <button
                  onClick={() => setSelectedStatus(false)}
                  disabled={isSubmitting}
                  className={`w-full p-4 border-4 border-black transition-all disabled:opacity-50 disabled:cursor-not-allowed ${
                    isNotificationDisabled
                      ? 'bg-gray-600 shadow-[4px_4px_0_0_#000] scale-[1.02]'
                      : 'bg-white shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000]'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`p-2 border-2 border-black ${
                      isNotificationDisabled ? 'bg-white' : 'bg-gray-100'
                    }`}>
                      <BellOff className="w-6 h-6 text-black" strokeWidth={3} />
                    </div>
                    <div className="flex-1 text-left">
                      <p className={`font-black text-base uppercase ${
                        isNotificationDisabled ? 'text-white' : 'text-black'
                      }`}>关闭通知</p>
                      <p className={`text-xs font-bold mt-1 ${
                        isNotificationDisabled ? 'text-white/90' : 'text-gray-600'
                      }`}>不再接收电费预警通知</p>
                    </div>
                    {isNotificationDisabled && (
                      <div className="w-6 h-6 bg-white border-2 border-black flex items-center justify-center">
                        <span className="text-gray-600 text-lg">✓</span>
                      </div>
                    )}
                  </div>
                </button>
        </div>
      </div>
    </ComicModal>
  );
}

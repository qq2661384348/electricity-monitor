/**
 * 算数验证码弹窗组件
 * Comic风格设计，与现有UI保持一致
 */

import { useState, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { RefreshCw } from 'lucide-react';
import { captchaService } from '@/services/captchaService';
import { ComicModal } from '@/components/ui/comic-modal';
import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/utils';

interface CaptchaModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess: (token: string) => void;
}

/**
 * 算数验证码Modal
 * 使用ComicModal组件保持风格一致
 */
export function CaptchaModal({ isOpen, onClose, onSuccess }: CaptchaModalProps) {
  const [captchaId, setCaptchaId] = useState<string>('');
  const [captchaImage, setCaptchaImage] = useState<string>('');
  const [userAnswer, setUserAnswer] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const inputRef = useRef<HTMLInputElement>(null);

  // 加载验证码
  const loadCaptcha = async () => {
    setIsRefreshing(true);
    setError('');
    setUserAnswer('');
    
    try {
      const response = await captchaService.generateMathCaptcha();
      setCaptchaId(response.data.id);
      setCaptchaImage(response.data.url);
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取验证码失败');
    } finally {
      setIsRefreshing(false);
    }
  };

  // 提交验证
  const handleSubmit = async () => {
    if (!userAnswer.trim()) {
      setError('请输入答案');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const response = await captchaService.verifyCaptcha({
        id: captchaId,
        key: userAnswer,
        type: 'math',
      });

      if (response.success && response.token) {
        // 验证成功
        onSuccess(response.token);
        onClose();
      } else {
        // 验证失败，自动刷新新验证码
        setError(response.message || '验证失败');
        await loadCaptcha();
        // 自动聚焦输入框
        setTimeout(() => inputRef.current?.focus(), 100);
      }
    } catch {
      setError('验证服务暂时不可用');
    } finally {
      setIsLoading(false);
    }
  };

  // 键盘事件处理
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !isLoading) {
      handleSubmit();
    }
  };

  // Modal打开时加载验证码
  useEffect(() => {
    if (isOpen) {
      loadCaptcha();
      // 延迟聚焦，等待动画完成
      setTimeout(() => inputRef.current?.focus(), 300);
    } else {
      // 关闭时清理状态
      setUserAnswer('');
      setError('');
      setCaptchaId('');
      setCaptchaImage('');
    }
  }, [isOpen]);

  return (
    <ComicModal
      isOpen={isOpen}
      onClose={onClose}
      size="sm"
      title="安全验证"
      showCloseButton
      overflowVisible
      footer={
        <div className="flex gap-3 w-full">
          <Button
            onClick={onClose}
            variant="secondary"
            size="lg"
            disabled={isLoading}
            className="flex-1"
          >
            取消
          </Button>
          <Button
            onClick={handleSubmit}
            variant="primary"
            size="lg"
            isLoading={isLoading}
            disabled={isLoading || !userAnswer.trim() || isRefreshing}
            className="flex-1"
          >
            {isLoading ? '验证中...' : '确认'}
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        {/* 说明文字 */}
        <div className="text-center">
          <p className="text-sm font-bold text-gray-700">
            请计算下面的算式并输入答案
          </p>
        </div>

        {/* 验证码图片区域 */}
        <div className="relative">
          <div className={cn(
            "relative bg-white border-4 border-black shadow-[4px_4px_0_0_#000] p-4",
            "flex items-center justify-center min-h-[120px]",
            isRefreshing && "opacity-50"
          )}>
            {captchaImage ? (
              <img 
                src={captchaImage} 
                alt="验证码"
                className="max-w-full h-auto"
              />
            ) : (
              <div className="flex items-center justify-center h-[100px]">
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
                >
                  <RefreshCw className="w-8 h-8 text-gray-400" />
                </motion.div>
              </div>
            )}
          </div>

          {/* 刷新按钮 */}
          <motion.button
            onClick={loadCaptcha}
            disabled={isRefreshing || isLoading}
            className={cn(
              "absolute -top-2 -right-2 w-10 h-10",
              "bg-brand-secondary border-2 border-black shadow-[2px_2px_0_0_#000]",
              "flex items-center justify-center",
              "hover:shadow-[3px_3px_0_0_#000] hover:-translate-x-px hover:-translate-y-px",
              "transition-all disabled:opacity-50 disabled:cursor-not-allowed",
              "z-10"
            )}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
          >
            <RefreshCw 
              className={cn(
                "w-5 h-5 text-black",
                isRefreshing && "animate-spin"
              )} 
              strokeWidth={3} 
            />
          </motion.button>
        </div>

        {/* 输入框 */}
        <div>
          <input
            ref={inputRef}
            type="text"
            value={userAnswer}
            onChange={(e) => {
              // 只允许输入数字和负号
              const value = e.target.value.replaceAll(/[^0-9-]/g, '');
              setUserAnswer(value);
            }}
            onKeyDown={handleKeyDown}
            placeholder="请输入计算结果"
            disabled={isLoading || isRefreshing}
            className={cn(
              "w-full px-4 py-3",
              "bg-white border-4 border-black shadow-[4px_4px_0_0_#000]",
              "text-2xl font-black text-center tracking-wider",
              "placeholder:text-gray-400 placeholder:font-normal placeholder:text-base",
              "focus:outline-none focus:shadow-[6px_6px_0_0_#000]",
              "disabled:opacity-50 disabled:cursor-not-allowed",
              "transition-all"
            )}
            maxLength={10}
          />
        </div>

        {/* 错误提示 */}
        {error && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            className="bg-status-danger text-white border-2 border-black shadow-[2px_2px_0_0_#000] p-3"
          >
            <p className="text-sm font-bold text-center">{error}</p>
          </motion.div>
        )}

        {/* 提示文字 */}
        <div className="text-center">
          <p className="text-xs text-gray-600">
            验证码2分钟内有效，错误将自动刷新
          </p>
        </div>
      </div>
    </ComicModal>
  );
}

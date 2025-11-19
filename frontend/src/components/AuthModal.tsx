import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Loader2, Zap } from 'lucide-react';
import { authApi } from '@/services/api';
import { useAuthStore } from '@/stores/authStore';
import { getMarvelQuote } from '@/lib/utils';
import { CaptchaModal } from '@/components/CaptchaModal';

interface AuthModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess?: () => void;
}

export function AuthModal({ isOpen, onClose, onSuccess }: AuthModalProps) {
  const [qqNumber, setQqNumber] = useState('');
  const [code, setCode] = useState('');
  const [countdown, setCountdown] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [quote] = useState(getMarvelQuote());
  const [showCaptcha, setShowCaptcha] = useState(false);
  
  const { login } = useAuthStore();

  // 倒计时
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // 点击发送验证码，先弹出算数验证码
  const handleSendCode = () => {
    if (!qqNumber || qqNumber.length < 5) {
      setError('请输入有效的QQ号');
      return;
    }
    setShowCaptcha(true);
  };

  // 验证码验证成功后的回调
  const handleCaptchaSuccess = async (token: string) => {
    setShowCaptcha(false);
    
    setIsLoading(true);
    setError('');

    try {
      await authApi.sendVerificationCode(qqNumber, token);
      setCountdown(60);
      setError('');
    } catch {
      setError('发送失败，请稍后重试');
    } finally {
      setIsLoading(false);
    }
  };

  // 验证登录
  const handleLogin = async () => {
    if (!qqNumber || code?.length !== 6) {
      setError('请输入完整信息');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const response = await authApi.verifyAndLogin(qqNumber, code);
      login(response.access_token, response.user);
      onSuccess?.();
      onClose();
    } catch {
      setError('验证失败，请检查验证码');
    } finally {
      setIsLoading(false);
    }
  };

  // 键盘回车提交
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !isLoading) {
      if (code.length === 6) {
        handleLogin();
      } else if (qqNumber && countdown === 0) {
        handleSendCode();
      }
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          {/* 背景遮罩 */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/90 backdrop-blur-sm"
            onClick={onClose}
          />

          {/* 模态框内容 - 漫画对话框风格 */}
          <motion.div
            initial={{ opacity: 0, scale: 0.5 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.5 }}
            transition={{ type: 'spring', damping: 15 }}
            className="relative w-full max-w-md"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative bg-white p-8 border-4 border-black shadow-[8px_8px_0_0_#000]">
              {/* 装饰元素：漫画点阵角标 */}
              <div className="absolute -top-4 -right-4 w-12 h-12 bg-brand-secondary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />
              <div className="absolute -bottom-4 -left-4 w-8 h-8 bg-brand-primary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />

              {/* 关闭按钮 */}
              <button
                onClick={onClose}
                className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center bg-black text-white border-2 border-black hover:bg-brand-primary hover:scale-110 transition-all z-50 shadow-[2px_2px_0_0_#666]"
              >
                <X size={20} strokeWidth={3} />
              </button>

              {/* 标题 */}
              <div className="relative z-10 text-center mb-8">
                <div className="inline-block mb-4 p-3 bg-brand-secondary border-2 border-black shadow-[4px_4px_0_0_#000] rounded-full">
                  <Zap className="w-10 h-10 text-black" strokeWidth={3} />
                </div>
                <h2 className="text-4xl font-black uppercase italic tracking-tighter text-black transform -skew-x-6" style={{ textShadow: '2px 2px 0 #0EA5E9' }}>
                  身份验证
                </h2>
                <p className="text-black font-bold bg-brand-secondary inline-block px-2 transform -rotate-1 mt-2 border border-black text-xs">
                  ACCESS RESTRICTED
                </p>
              </div>

              {/* 表单 */}
              <div className="relative z-10 space-y-6">
                {/* QQ号输入 */}
                <div>
                  <label htmlFor="auth-qq" className="comic-label">
                    QQ号码
                  </label>
                  <input
                    id="auth-qq"
                    type="text"
                    value={qqNumber}
                    onChange={(e) => setQqNumber(e.target.value.replaceAll(/\D/g, ''))}
                    onKeyDown={handleKeyDown}
                    placeholder="输入QQ号..."
                    className="comic-input focus:ring-brand-primary"
                    disabled={isLoading}
                  />
                </div>

                {/* 发送验证码按钮 */}
                <button
                  onClick={handleSendCode}
                  disabled={isLoading || countdown > 0 || !qqNumber}
                  className="comic-button w-full bg-brand-accent text-white text-sm py-2 shadow-[3px_3px_0_0_#000]"
                >
                  {countdown > 0 ? `${countdown}秒后重试` : '发送验证码'}
                </button>

                {/* 验证码输入 */}
                <div>
                  <label htmlFor="auth-code" className="comic-label">
                    验证码
                  </label>
                  <input
                    id="auth-code"
                    type="text"
                    value={code}
                    onChange={(e) => setCode(e.target.value.replaceAll(/\D/g, '').slice(0, 6))}
                    onKeyDown={handleKeyDown}
                    placeholder="######"
                    className="comic-input text-center text-3xl tracking-[0.5em] font-black focus:ring-brand-primary"
                    disabled={isLoading}
                    maxLength={6}
                  />
                </div>

                {/* 错误提示 */}
                {error && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    className="bg-brand-danger text-white font-bold px-4 py-2 border-2 border-black text-center text-sm shadow-[2px_2px_0_0_#000]"
                  >
                    {error.toUpperCase()}!
                  </motion.div>
                )}

                {/* 登录按钮 */}
                <button
                  onClick={handleLogin}
                  disabled={isLoading || code?.length !== 6}
                  className="comic-button w-full text-xl py-4 mt-4 bg-brand-primary hover:bg-sky-400"
                >
                  <span className="flex items-center justify-center gap-2">
                    {isLoading && <Loader2 className="animate-spin" size={24} />}
                    {isLoading ? '验证中...' : '确认进入'}
                  </span>
                </button>
              </div>

              {/* 英雄名言 (漫画气泡样式) */}
              <div className="relative z-10 mt-8 p-4 bg-white border-2 border-black shadow-[4px_4px_0_0_#000]">
                <p className="text-black text-sm font-bold italic text-center font-serif">
                  "{quote.text}"
                </p>
                <p className="text-right text-xs font-black text-brand-primary mt-2 uppercase">
                  — {quote.author}
                </p>
              </div>
            </div>
          </motion.div>
        </div>
      )}

      {/* 验证码弹窗 */}
      <CaptchaModal
        isOpen={showCaptcha}
        onClose={() => setShowCaptcha(false)}
        onSuccess={handleCaptchaSuccess}
      />
    </AnimatePresence>
  );
}

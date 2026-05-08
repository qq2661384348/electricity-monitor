import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Mail, MessageCircle, X, Loader2, Zap } from 'lucide-react';
import { useAuthLoginFlow } from '@/features/auth-login';
import { getMarvelQuote } from '@/lib/utils';
import { CaptchaModal } from '@/components/CaptchaModal';

interface AuthModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess?: () => void;
}

export function AuthModal({ isOpen, onClose, onSuccess }: AuthModalProps) {
  const [quote] = useState(getMarvelQuote());
  const authFlow = useAuthLoginFlow({
    onLoginSuccess: () => {
      onSuccess?.();
      onClose();
    },
  });

  return (
    <>
      <AnimatePresence>
        {isOpen && (
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            {/* 背景遮罩 */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="absolute inset-0 bg-black/90 backdrop-blur-sm"
              onClick={() => {
                // 当有错误时，不允许点击背景关闭，确保用户看到错误提示
                if (!authFlow.error) {
                  onClose();
                }
              }}
            />

            {/* 模态框内容 - 漫画对话框风格 */}
            <motion.div
              initial={{ opacity: 0, scale: 0.5 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.5 }}
              transition={{ type: 'spring', damping: 15 }}
              className="relative w-full max-w-[calc(100%-1rem)] sm:max-w-md"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="relative bg-white p-4 sm:p-6 md:p-8 border-4 border-black shadow-[4px_4px_0_0_#000] sm:shadow-[6px_6px_0_0_#000] md:shadow-[8px_8px_0_0_#000]">
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

                {/* 标题 - 响应式设计 */}
                <div className="relative z-10 text-center mb-4 sm:mb-6 md:mb-8">
                  <div className="inline-block mb-3 sm:mb-4 p-2 sm:p-3 bg-brand-secondary border-2 border-black shadow-[4px_4px_0_0_#000] rounded-full">
                    <Zap className="w-8 h-8 sm:w-10 sm:h-10 text-black" strokeWidth={3} />
                  </div>
                  <h2 className="text-2xl sm:text-3xl md:text-4xl font-black uppercase italic tracking-tighter text-black transform -skew-x-6" style={{ textShadow: '2px 2px 0 #0EA5E9' }}>
                    身份验证
                  </h2>
                  <p className="text-black font-bold bg-brand-secondary inline-block px-2 transform -rotate-1 mt-2 border border-black text-xs">
                    ACCESS RESTRICTED
                  </p>
                </div>

                {/* 表单 - 响应式间距 */}
                <div className="relative z-10 space-y-4 sm:space-y-6">
                  <div
                    role="tablist"
                    aria-label="选择登录方式"
                    className="grid grid-cols-2 gap-2"
                  >
                    <button
                      type="button"
                      role="tab"
                      aria-selected={authFlow.loginMode === 'qq'}
                      onClick={() => authFlow.switchLoginMode('qq')}
                      className={`flex items-center justify-center gap-2 border-2 border-black px-3 py-2 text-xs sm:text-sm font-black shadow-[2px_2px_0_0_#000] transition-all ${
                        authFlow.loginMode === 'qq'
                          ? 'bg-brand-primary text-white'
                          : 'bg-white text-black hover:bg-yellow-100'
                      }`}
                    >
                      <MessageCircle className="h-4 w-4" />
                      QQ机器人
                    </button>
                    <button
                      type="button"
                      role="tab"
                      aria-selected={authFlow.loginMode === 'email'}
                      aria-disabled={!authFlow.emailLoginAvailable}
                      disabled={!authFlow.emailLoginAvailable}
                      title={authFlow.emailLoginAvailable ? '邮箱登录' : '邮件服务未配置'}
                      onClick={() => authFlow.switchLoginMode('email')}
                      className={`flex items-center justify-center gap-2 border-2 border-black px-3 py-2 text-xs sm:text-sm font-black shadow-[2px_2px_0_0_#000] transition-all ${
                        authFlow.loginMode === 'email'
                          ? 'bg-brand-primary text-white'
                          : 'bg-white text-black hover:bg-yellow-100'
                      } disabled:cursor-not-allowed disabled:bg-gray-200 disabled:text-gray-500`}
                    >
                      <Mail className="h-4 w-4" />
                      邮箱登录
                    </button>
                  </div>

                  {/* 登录标识输入 */}
                  <div>
                    <label htmlFor="auth-identifier" className="comic-label">
                      {authFlow.currentIdentifierLabel}
                    </label>
                    <input
                      id="auth-identifier"
                      type={authFlow.loginMode === 'email' ? 'email' : 'text'}
                      value={authFlow.currentIdentifier}
                      onChange={(e) => authFlow.updateIdentifier(e.target.value)}
                      onKeyDown={authFlow.handleKeyDown}
                      placeholder={authFlow.loginMode === 'qq' ? '输入QQ号...' : '输入邮箱地址...'}
                      className="comic-input focus:ring-brand-primary"
                      disabled={authFlow.isLoading}
                    />
                  </div>

                  {/* 发送验证码按钮 */}
                  <button
                    onClick={authFlow.requestVerificationCode}
                    disabled={
                      authFlow.isLoading ||
                      authFlow.countdown > 0 ||
                      !authFlow.identifierValid
                    }
                    className="comic-button w-full bg-brand-accent text-white text-sm py-2 shadow-[3px_3px_0_0_#000]"
                  >
                    {authFlow.countdown > 0 ? `${authFlow.countdown}秒后重试` : '发送验证码'}
                  </button>

                  {/* 验证码输入 */}
                  <div>
                    <label htmlFor="auth-code" className="comic-label">
                      验证码
                    </label>
                    <input
                      id="auth-code"
                      type="text"
                      value={authFlow.code}
                      onChange={(e) => authFlow.updateCode(e.target.value)}
                      onKeyDown={authFlow.handleKeyDown}
                      placeholder={authFlow.codePlaceholder}
                      className="comic-input text-center text-xl sm:text-2xl md:text-3xl tracking-[0.12em] font-black focus:ring-brand-primary"
                      disabled={authFlow.isLoading}
                      maxLength={authFlow.codeLength}
                    />
                  </div>

                  {/* 错误提示 */}
                  {authFlow.error && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      className="bg-status-danger text-black font-black px-4 py-3 border-4 border-black text-center text-base shadow-[4px_4px_0_0_#000]"
                    >
                      {authFlow.error}
                    </motion.div>
                  )}

                  {/* 登录按钮 */}
                  <button
                    onClick={authFlow.submitLogin}
                    disabled={
                      authFlow.isLoading ||
                      !authFlow.identifierValid ||
                      authFlow.code.length !== authFlow.codeLength
                    }
                    className="comic-button w-full text-xl py-4 mt-4 bg-brand-primary hover:bg-sky-400"
                  >
                    <span className="flex items-center justify-center gap-2">
                      {authFlow.isLoading && <Loader2 className="animate-spin" size={24} />}
                      {authFlow.isLoading ? '验证中...' : '确认进入'}
                    </span>
                  </button>
                </div>

                {/* 英雄名言 (漫画气泡样式) - 移动端隐藏 */}
                <div className="relative z-10 mt-4 sm:mt-6 md:mt-8 p-3 sm:p-4 bg-white border-2 border-black shadow-[4px_4px_0_0_#000] hidden sm:block">
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
      </AnimatePresence>

      {/* 验证码弹窗 - 移到外部独立管理，避免 AnimatePresence 嵌套 */}
      <CaptchaModal
        isOpen={authFlow.showCaptcha}
        onClose={authFlow.closeCaptcha}
        onSuccess={authFlow.handleCaptchaSuccess}
        publicConfig={authFlow.publicConfig}
      />
    </>
  );
}

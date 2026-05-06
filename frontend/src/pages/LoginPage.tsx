import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Mail, MessageCircle, Zap } from 'lucide-react';
import { AxiosError } from 'axios';
import { useAuthStore } from '@/stores/authStore';
import {
  authApi,
  isValidLoginIdentifier,
  loginModeLabel,
  normalizeLoginIdentifier,
  sanitizeLoginIdentifier,
} from '@/features/auth-login';
import { fallbackPublicConfig } from '@/entities/public-config';
import { usePublicConfig } from '@/features/public-config';
import { Input } from '@/components/ui/Input';
import { Button } from '@/components/ui/Button';
import { CaptchaModal } from '@/components/CaptchaModal';
import type { LoginMode } from '@/types';
import { motion } from 'framer-motion';

// 随机漫威名言
const quotes = [
  { text: "能力越大，责任越大", author: "Spider-Man" },
  { text: "我就是钢铁侠", author: "Iron Man" },
  { text: "复仇者，集合！", author: "Captain America" },
  { text: "瓦坎达万岁！", author: "Black Panther" },
  { text: "我不仅是神，还是个好人", author: "Thor" },
];

export default function LoginPage() {
  const navigate = useNavigate();
  const { login, isAuthenticated } = useAuthStore();
  const { data: publicConfig } = usePublicConfig();
  const resolvedPublicConfig = publicConfig ?? fallbackPublicConfig;
  const codeLength = resolvedPublicConfig.verification.code_length;
  const codePlaceholder = `${codeLength}位验证码`;
  const emailLoginAvailable =
    resolvedPublicConfig.auth.email_login_enabled &&
    resolvedPublicConfig.auth.login_modes.includes('email');

  const [loginMode, setLoginMode] = useState<LoginMode>('qq');
  const [qqNumber, setQqNumber] = useState('');
  const [email, setEmail] = useState('');
  const [code, setCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [quote, setQuote] = useState(quotes[0]);
  const [showCaptcha, setShowCaptcha] = useState(false);

  // 如果已登录，重定向到首页
  useEffect(() => {
    if (isAuthenticated) {
      navigate('/', { replace: true });
    }
  }, [isAuthenticated, navigate]);

  useEffect(() => {
    setQuote(quotes[Math.floor(Math.random() * quotes.length)]);
  }, []);

  useEffect(() => {
    let timer: NodeJS.Timeout;
    if (countdown > 0) {
      timer = setTimeout(() => setCountdown(c => c - 1), 1000);
    }
    return () => clearTimeout(timer);
  }, [countdown]);

  const formatUserNotFriendMessage = () => {
    const botQQ = resolvedPublicConfig.notification.qq_bot_public_qq_number.trim();
    const adminQQ = resolvedPublicConfig.notification.admin_qq_number.trim();
    const botText = botQQ || '请联系管理员获取机器人QQ号';
    const adminText = adminQQ || '请联系管理员';
    return `请先添加机器人QQ号：${botText}。遇到问题请联系管理员：${adminText}。`;
  };

  const currentIdentifier = loginMode === 'qq' ? qqNumber : email;
  const currentIdentifierLabel = loginModeLabel(loginMode);
  const normalizedIdentifier = normalizeLoginIdentifier(loginMode, currentIdentifier);
  const identifierValid =
    loginMode === 'email' && !emailLoginAvailable
      ? false
      : isValidLoginIdentifier(loginMode, currentIdentifier);

  const switchLoginMode = (mode: LoginMode) => {
    if (mode === 'email' && !emailLoginAvailable) {
      setError('邮箱登录暂未启用');
      return;
    }

    setLoginMode(mode);
    setCode('');
    setError(null);
  };

  // 点击发送验证码，先弹出算数验证码
  const handleSendCode = () => {
    if (!identifierValid) {
      setError(`请输入有效的${currentIdentifierLabel}`);
      return;
    }
    setShowCaptcha(true);
  };

  // 验证码验证成功后的回调
  const handleCaptchaSuccess = async (token: string) => {
    setShowCaptcha(false);
    
    setIsLoading(true);
    setError(null);
    try {
      await authApi.sendVerificationCode(loginMode, normalizedIdentifier, token);
      setCountdown(60);
    } catch (err: unknown) {
      let errorMessage = '发送失败，请稍后重试';
      if (err instanceof AxiosError && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        // 识别 USER_NOT_FRIEND 错误
        if (data.error === 'USER_NOT_FRIEND') {
          errorMessage = formatUserNotFriendMessage();
        } else if (typeof data.detail === 'string') {
          errorMessage = data.detail;
        } else if (typeof data.message === 'string') {
          errorMessage = data.message;
        }
      }
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogin = async () => {
    if (!identifierValid || code.length !== codeLength) {
      setError(`请输入有效的${currentIdentifierLabel}和${codeLength}位验证码`);
      return;
    }

    setIsLoading(true);
    setError(null);
    try {
      const response = await authApi.verifyAndLogin(loginMode, normalizedIdentifier, code);
      login(response.access_token, response.user);
      // 导航将由 useEffect 自动处理
    } catch (err: unknown) {
      let errorMessage = '验证失败，请检查验证码';
      if (err instanceof AxiosError && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        if (typeof data.detail === 'string') {
          errorMessage = data.detail;
        }
      }
      setError(errorMessage);
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-brand-dark flex items-center justify-center p-4 bg-[radial-gradient(#1e293b_15%,transparent_16%)] bg-size-[20px_20px]">
       <motion.div 
         initial={{ scale: 0.9, opacity: 0 }}
         animate={{ scale: 1, opacity: 1 }}
         className="relative bg-white p-4 sm:p-6 md:p-8 border-4 border-black shadow-[4px_4px_0_0_#000] sm:shadow-[6px_6px_0_0_#000] md:shadow-[8px_8px_0_0_#000] max-w-[calc(100%-1rem)] sm:max-w-md w-full"
       >
          {/* 装饰元素 */}
          <div className="absolute -top-4 -right-4 w-12 h-12 bg-brand-secondary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />
          <div className="absolute -bottom-4 -left-4 w-8 h-8 bg-brand-primary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />

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
                aria-selected={loginMode === 'qq'}
                onClick={() => switchLoginMode('qq')}
                className={`flex items-center justify-center gap-2 border-2 border-black px-3 py-2 text-xs sm:text-sm font-black shadow-[2px_2px_0_0_#000] transition-all ${
                  loginMode === 'qq'
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
                aria-selected={loginMode === 'email'}
                aria-disabled={!emailLoginAvailable}
                disabled={!emailLoginAvailable}
                title={emailLoginAvailable ? '邮箱登录' : '邮件服务未配置'}
                onClick={() => switchLoginMode('email')}
                className={`flex items-center justify-center gap-2 border-2 border-black px-3 py-2 text-xs sm:text-sm font-black shadow-[2px_2px_0_0_#000] transition-all ${
                  loginMode === 'email'
                    ? 'bg-brand-primary text-white'
                    : 'bg-white text-black hover:bg-yellow-100'
                } disabled:cursor-not-allowed disabled:bg-gray-200 disabled:text-gray-500`}
              >
                <Mail className="h-4 w-4" />
                邮箱登录
              </button>
            </div>

            <div>
              <label htmlFor="login-identifier" className="comic-label">
                {currentIdentifierLabel}
              </label>
              <Input
                id="login-identifier"
                type={loginMode === 'email' ? 'email' : 'text'}
                value={currentIdentifier}
                onChange={(e) => {
                  const value = sanitizeLoginIdentifier(loginMode, e.target.value);
                  if (loginMode === 'qq') {
                    setQqNumber(value);
                  } else {
                    setEmail(value);
                  }
                }}
                placeholder={loginMode === 'qq' ? '输入QQ号...' : '输入邮箱地址...'}
                disabled={isLoading}
              />
            </div>

            <Button
              onClick={handleSendCode}
              disabled={isLoading || countdown > 0 || !identifierValid}
              fullWidth
              variant="accent"
              size="sm"
            >
              {countdown > 0 ? `${countdown}秒后重试` : '发送验证码'}
            </Button>

            <div>
              <label htmlFor="verification-code" className="comic-label">
                验证码
              </label>
              <Input
                id="verification-code"
                type="text"
                value={code}
                onChange={(e) => setCode(e.target.value.replaceAll(/\D/g, '').slice(0, codeLength))}
                placeholder={codePlaceholder}
                className="text-center text-xl sm:text-2xl md:text-3xl tracking-[0.12em] font-black"
                disabled={isLoading}
                maxLength={codeLength}
              />
            </div>

            {error && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                className="bg-status-danger text-white font-bold px-4 py-2 border-2 border-black text-center text-sm shadow-[2px_2px_0_0_#000]"
              >
                {error}
              </motion.div>
            )}

            <Button
              onClick={handleLogin}
              disabled={isLoading || !identifierValid || code.length !== codeLength}
              fullWidth
              size="lg"
              isLoading={isLoading}
            >
              确认进入
            </Button>
          </div>

          {/* 名言 - 移动端隐藏 */}
          <div className="relative z-10 mt-4 sm:mt-6 md:mt-8 p-3 sm:p-4 bg-white border-2 border-black shadow-[4px_4px_0_0_#000] hidden sm:block">
            <p className="text-black text-sm font-bold italic text-center font-serif">
              "{quote.text}"
            </p>
            <p className="text-right text-xs font-black text-brand-primary mt-2 uppercase">
              — {quote.author}
            </p>
          </div>
       </motion.div>

       {/* 验证码弹窗 */}
       <CaptchaModal
         isOpen={showCaptcha}
         onClose={() => setShowCaptcha(false)}
         onSuccess={handleCaptchaSuccess}
         publicConfig={resolvedPublicConfig}
       />
    </div>
  );
}

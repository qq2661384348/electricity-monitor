import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Zap } from 'lucide-react';
import { AxiosError } from 'axios';
import { useAuthStore } from '@/stores/authStore';
import { authApi } from '@/services/api';
import { Input } from '@/components/ui/Input';
import { Button } from '@/components/ui/Button';
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
  
  const [qqNumber, setQqNumber] = useState('');
  const [code, setCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [quote, setQuote] = useState(quotes[0]);

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

  const handleSendCode = async () => {
    if (!qqNumber) return;
    
    setIsLoading(true);
    setError(null);
    try {
      await authApi.sendVerificationCode(qqNumber);
      setCountdown(60);
    } catch (err: unknown) {
      let errorMessage = '发送失败，请稍后重试';
      if (err instanceof AxiosError && err.response?.data) {
        const data = err.response.data as Record<string, unknown>;
        if (typeof data.detail === 'string') {
          errorMessage = data.detail;
        }
      }
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogin = async () => {
    if (!qqNumber || !code) return;

    setIsLoading(true);
    setError(null);
    try {
      const response = await authApi.verifyAndLogin(qqNumber, code);
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
         className="relative bg-white p-8 border-4 border-black shadow-[8px_8px_0_0_#000] max-w-md w-full"
       >
          {/* 装饰元素 */}
          <div className="absolute -top-4 -right-4 w-12 h-12 bg-brand-secondary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />
          <div className="absolute -bottom-4 -left-4 w-8 h-8 bg-brand-primary border-2 border-black z-20 shadow-[4px_4px_0_0_#000]" />

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
            <div>
              <label htmlFor="qq-number" className="comic-label">
                QQ号码
              </label>
              <Input
                id="qq-number"
                type="text"
                value={qqNumber}
                onChange={(e) => setQqNumber(e.target.value.replaceAll(/\D/g, ''))}
                placeholder="输入QQ号..."
                disabled={isLoading}
              />
            </div>

            <Button
              onClick={handleSendCode}
              disabled={isLoading || countdown > 0 || !qqNumber}
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
                onChange={(e) => setCode(e.target.value.replaceAll(/\D/g, '').slice(0, 6))}
                placeholder="######"
                className="text-center text-3xl tracking-[0.5em] font-black"
                disabled={isLoading}
                maxLength={6}
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
              disabled={isLoading || code?.length !== 6}
              fullWidth
              size="lg"
              isLoading={isLoading}
            >
              确认进入
            </Button>
          </div>

          {/* 名言 */}
          <div className="relative z-10 mt-8 p-4 bg-white border-2 border-black shadow-[4px_4px_0_0_#000]">
            <p className="text-black text-sm font-bold italic text-center font-serif">
              "{quote.text}"
            </p>
            <p className="text-right text-xs font-black text-brand-primary mt-2 uppercase">
              — {quote.author}
            </p>
          </div>
       </motion.div>
    </div>
  );
}

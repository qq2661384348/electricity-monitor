import { useState } from 'react';
import { motion } from 'framer-motion';
import { Zap, User, Bell, LogOut } from 'lucide-react';
import { AnnouncementModal } from '@/components/AnnouncementModal';
import { UserInfoModal } from '@/components/UserInfoModal';
import { useAuthStore } from '@/stores/authStore';

interface NavbarProps {
  readonly onLoginClick: () => void;
}

export function Navbar({ onLoginClick }: NavbarProps) {
  const { user, isAuthenticated, logout } = useAuthStore();
  const [isAnnouncementOpen, setIsAnnouncementOpen] = useState(false);
  const [isUserInfoOpen, setIsUserInfoOpen] = useState(false);

  return (
    <motion.nav
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      className="fixed top-0 left-0 right-0 z-40 bg-linear-to-r from-[#ffe173] via-[#ffd966] to-[#ffcc33] border-b-4 border-black shadow-[0_4px_0_0_#000]"
    >
      <div className="max-w-7xl mx-auto px-3 sm:px-4 py-2 sm:py-3 flex items-center justify-between">
        {/* Logo - 响应式设计 */}
        <div className="flex items-center gap-2 sm:gap-3 group cursor-pointer">
          <div className="relative p-1.5 sm:p-2 bg-brand-primary border-2 border-black transform -rotate-3 group-hover:rotate-0 transition-transform shadow-[2px_2px_0_0_#000]">
            <Zap className="w-5 h-5 sm:w-6 sm:h-6 text-black fill-current" />
          </div>
          <div className="flex flex-col leading-tight text-center">
            <span className="text-xl sm:text-2xl md:text-3xl tracking-wide text-black transform skew-x-[-5deg]" style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}>
              电力监控
            </span>
            <span className="text-2xl sm:text-3xl md:text-4xl tracking-wide text-brand-primary transform skew-x-[-5deg]" style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}>
              系统
            </span>
          </div>
        </div>

        {/* 用户区域 - 响应式间距 */}
        <div className="flex items-center gap-2 sm:gap-4">
          {/* 通知按钮 */}
          {isAuthenticated && (
            <button
              className="relative p-2 border-2 border-black hover:border-brand-primary rounded-none transition-all hover:bg-yellow-200 shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000]"
              onClick={() => setIsAnnouncementOpen(true)}
              aria-label="查看公告"
            >
              <Bell className="w-6 h-6 text-black" />
              <span className="absolute top-1 right-1 w-3 h-3 bg-status-danger border-2 border-black rounded-full" />
            </button>
          )}

          {/* 用户信息/登录按钮 */}
          {isAuthenticated && user ? (
            <div className="flex items-center gap-4">
              {/* 用户信息 - 移动端简化显示 */}
              <div className="text-right">
                <div className="text-xs sm:text-sm font-black text-black uppercase tracking-wider">{user.qq_number}</div>
                <div className="hidden sm:inline-block text-xs font-bold text-white bg-brand-primary px-2 py-1 border-2 border-black shadow-[2px_2px_0_0_#000]">
                  {user.role === 'admin' ? 'ADMIN' : 'USER'}
                </div>
              </div>
              <div className="flex items-center gap-3">
                <button
                  type="button"
                  onClick={() => setIsUserInfoOpen(true)}
                  className="w-10 h-10 bg-white border-2 border-black flex items-center justify-center shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000] hover:-translate-y-0.5 transition-all"
                  aria-label="查看个人信息"
                >
                  <User className="w-6 h-6 text-black" />
                </button>
                <button
                  onClick={logout}
                  className="p-2 bg-status-danger border-2 border-black text-white hover:translate-y-[-2px] hover:shadow-[3px_3px_0_0_#000] shadow-[2px_2px_0_0_#000] transition-all"
                  title="退出登录"
                >
                  <LogOut className="w-5 h-5" />
                </button>
              </div>
            </div>
          ) : (
            <button
              onClick={onLoginClick}
              className="comic-button relative overflow-hidden bg-brand-primary text-white border-2 border-black shadow-[4px_4px_0_0_#000] hover:bg-brand-secondary hover:shadow-[6px_6px_0_0_#000] hover:-translate-y-1 transition-all"
            >
              <span className="relative z-10 font-black italic tracking-widest text-lg">登录</span>
              <div className="absolute inset-0 bg-[radial-gradient(rgba(0,0,0,0.2)_1px,transparent_1px)] bg-size-[4px_4px] opacity-30" />
            </button>
          )}
        </div>
      </div>
      {isAuthenticated && (
        <AnnouncementModal
          isOpen={isAnnouncementOpen}
          onClose={() => setIsAnnouncementOpen(false)}
        />
      )}
      {isAuthenticated && user && (
        <UserInfoModal
          isOpen={isUserInfoOpen}
          onClose={() => setIsUserInfoOpen(false)}
          qqNumber={user.qq_number}
          role={user.role}
        />
      )}
    </motion.nav>
  );
}

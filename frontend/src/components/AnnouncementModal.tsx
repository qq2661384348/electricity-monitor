import { Megaphone, MessageSquareHeart } from 'lucide-react';
import { ComicModal } from '@/components/ui/comic-modal';
import { fallbackPublicConfig } from '@/entities/public-config';
import { usePublicConfig } from '@/features/public-config';
import { CopyableQQNumber } from '@/components/CopyableQQNumber';

interface AnnouncementModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
}

interface InlineQQNumberProps {
  readonly value: string;
  readonly fallback: string;
}

function InlineQQNumber({ value, fallback }: InlineQQNumberProps) {
  const qqNumber = value.trim();
  if (qqNumber) {
    return <CopyableQQNumber value={qqNumber} />;
  }

  return <span className="font-black text-black">{fallback}</span>;
}

/**
 * AnnouncementModal - 使用新的 ComicModal 组件库重构
 * 
 * 代码量从 93 行减少到 ~50 行（减少 46%）
 * 无需手动管理动画、装饰元素等重复逻辑
 */
export function AnnouncementModal({ isOpen, onClose }: AnnouncementModalProps) {
  const { data: publicConfig } = usePublicConfig();
  const resolvedPublicConfig = publicConfig ?? fallbackPublicConfig;
  const adminQQ = resolvedPublicConfig.notification.admin_qq_number;

  return (
    <ComicModal
      isOpen={isOpen}
      onClose={onClose}
      size="lg"
      showCloseButton
      footer={
        <button
          onClick={onClose}
          className="w-full px-10 py-3 bg-white text-black font-black text-lg uppercase border-2 border-black shadow-[4px_4px_0_0_#000] hover:shadow-[5px_5px_0_0_#000] hover:-translate-y-0.5 transition-all"
        >
          我知道了
        </button>
      }
    >
      <div className="text-center space-y-3 sm:space-y-4">
        <div className="inline-flex items-center gap-3 px-4 py-2 bg-black text-white border-2 border-black shadow-[4px_4px_0_0_#000]">
          <Megaphone className="w-6 h-6" strokeWidth={3} />
          <span className="font-black text-lg tracking-widest uppercase">公告</span>
        </div>

        <h3
          className="text-2xl sm:text-3xl font-black uppercase italic text-black"
          style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}
        >
          系统通知
        </h3>

        <div className="relative bg-white/80 border-2 border-black shadow-[4px_4px_0_0_#000] p-3 sm:p-5 text-left">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-brand-primary border-2 border-black flex items-center justify-center shadow-[3px_3px_0_0_#000]">
              <MessageSquareHeart className="w-6 h-6 text-black" strokeWidth={3} />
            </div>
            <div>
              <p className="text-xs font-black uppercase tracking-widest text-gray-600">开源公告</p>
              <p className="text-base font-black text-black">项目源码已公开发布</p>
            </div>
          </div>

          <p className="text-sm font-bold text-gray-800 leading-relaxed">
            项目已开源到{' '}
            <a
              href="https://github.com/qq2661384348/electricity-monitor"
              target="_blank"
              rel="noreferrer"
              className="break-all font-black text-brand-primary underline decoration-2 underline-offset-4 hover:text-sky-600"
            >
              https://github.com/qq2661384348/electricity-monitor
            </a>
            。如果有问题及时联系管理员：
            <InlineQQNumber value={adminQQ} fallback="正在读取管理员QQ" />
            。
          </p>
        </div>
      </div>
    </ComicModal>
  );
}

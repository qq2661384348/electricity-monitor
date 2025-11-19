import { motion } from 'framer-motion';
import { useComicModalContext } from './ComicModalContext';
import { OVERLAY_ANIMATION } from './constants';
import type { ComicModalOverlayProps } from './types';

/**
 * ComicModalOverlay - 模态框背景遮罩层
 * 
 * 提供半透明黑色背景和模糊效果
 * 默认点击遮罩会关闭模态框
 * 
 * @example
 * ```tsx
 * <ComicModalOverlay />
 * <ComicModalOverlay closeOnClick={false} />
 * ```
 */
export function ComicModalOverlay({
  closeOnClick = true,
  className = '',
}: ComicModalOverlayProps) {
  const { onClose } = useComicModalContext();

  return (
    <motion.div
      {...OVERLAY_ANIMATION}
      className={`fixed inset-0 bg-black/80 backdrop-blur-sm ${className}`}
      onClick={closeOnClick ? onClose : undefined}
      aria-hidden="true"
    />
  );
}

ComicModalOverlay.displayName = 'ComicModalOverlay';

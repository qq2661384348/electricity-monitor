import { motion } from 'framer-motion';
import { useComicModalContext } from './ComicModalContext';
import { CONTENT_ANIMATION, MODAL_SIZE_CLASSES, HALFTONE_STYLE } from './constants';
import type { ComicModalContentProps } from './types';

/**
 * ComicModalContent - 模态框主内容容器
 * 
 * 包含漫画风格装饰：渐变背景、边框阴影、半调纹理、装饰色块
 * 
 * @example
 * ```tsx
 * <ComicModalContent>
 *   <ComicModalHeader>...</ComicModalHeader>
 *   <ComicModalBody>...</ComicModalBody>
 * </ComicModalContent>
 * ```
 */
export function ComicModalContent({
  children,
  className = '',
  stopPropagation = true,
}: Readonly<ComicModalContentProps>) {
  const { size, decorations } = useComicModalContext();

  return (
    <motion.div
      {...CONTENT_ANIMATION}
      className={`relative w-full ${MODAL_SIZE_CLASSES[size]} max-h-[90vh] p-4 ${className}`}
      onClick={stopPropagation ? (e) => e.stopPropagation() : undefined}
    >
      <div className="relative overflow-hidden p-8 border-4 border-black shadow-[10px_10px_0_0_#000] text-black bg-linear-to-br from-[#fff4c7] via-[#ffe173] to-[#ffc93c] h-full flex flex-col">
        {/* 漫画半调纹理 */}
        {decorations.halftone && (
          <div
            className="absolute inset-0 opacity-15 pointer-events-none"
            style={HALFTONE_STYLE}
          />
        )}

        {/* 装饰元素 - 右上角色块 */}
        {decorations.topRight && (
          <div className="absolute -top-4 -right-4 w-10 h-10 bg-brand-secondary border-2 border-black z-30 shadow-[4px_4px_0_0_#000]" />
        )}

        {/* 装饰元素 - 左下角色块 */}
        {decorations.bottomLeft && (
          <div className="absolute -bottom-4 -left-4 w-8 h-8 bg-brand-primary border-2 border-black z-30 shadow-[4px_4px_0_0_#000]" />
        )}

        {/* 子内容 */}
        {children}
      </div>
    </motion.div>
  );
}

ComicModalContent.displayName = 'ComicModalContent';

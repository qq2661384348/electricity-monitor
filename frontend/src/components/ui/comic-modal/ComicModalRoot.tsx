import { useMemo } from 'react';
import { AnimatePresence } from 'framer-motion';
import { ComicModalContext } from './ComicModalContext';
import { DEFAULT_DECORATIONS } from './constants';
import type { ComicModalRootProps } from './types';

/**
 * ComicModalRoot - 模态框根组件
 * 
 * 提供 Context 给所有子组件，管理模态框的整体状态
 * 使用 AnimatePresence 包装以支持退出动画
 * 
 * @example
 * ```tsx
 * <ComicModalRoot isOpen={isOpen} onClose={onClose} size="md">
 *   {// 子组件}
 * </ComicModalRoot>
 * ```
 */
export function ComicModalRoot({
  isOpen,
  onClose,
  size = 'md',
  decorations = DEFAULT_DECORATIONS,
  children,
}: Readonly<ComicModalRootProps>) {
  const contextValue = useMemo(
    () => ({
      isOpen,
      onClose,
      size,
      decorations: { ...DEFAULT_DECORATIONS, ...decorations },
    }),
    [isOpen, onClose, size, decorations]
  );

  return (
    <ComicModalContext.Provider value={contextValue}>
      <AnimatePresence mode="wait">
        {isOpen && children}
      </AnimatePresence>
    </ComicModalContext.Provider>
  );
}

ComicModalRoot.displayName = 'ComicModalRoot';

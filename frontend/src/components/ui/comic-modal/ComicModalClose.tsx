import { X } from 'lucide-react';
import { useComicModalContext } from './ComicModalContext';
import type { ComicModalCloseProps } from './types';

/**
 * ComicModalClose - 关闭按钮组件
 * 
 * 默认渲染为右上角的 X 按钮
 * 
 * @example
 * ```tsx
 * <ComicModalClose />
 * <ComicModalClose aria-label="关闭对话框" />
 * ```
 */
export function ComicModalClose({
  className = '',
  'aria-label': ariaLabel = '关闭',
}: ComicModalCloseProps) {
  const { onClose } = useComicModalContext();

  return (
    <button
      type="button"
      onClick={onClose}
      className={`absolute top-4 right-4 w-8 h-8 flex items-center justify-center bg-black text-white border-2 border-black hover:bg-brand-primary hover:scale-110 transition-all z-40 ${className}`}
      aria-label={ariaLabel}
    >
      <X size={16} strokeWidth={3} />
    </button>
  );
}

ComicModalClose.displayName = 'ComicModalClose';

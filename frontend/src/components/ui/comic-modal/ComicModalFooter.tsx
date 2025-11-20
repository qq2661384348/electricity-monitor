import type { ComicModalFooterProps } from './types';

/**
 * ComicModalFooter - 模态框底部区域
 * 
 * 通常用于放置操作按钮
 * 
 * @example
 * ```tsx
 * <ComicModalFooter>
 *   <Button onClick={onCancel}>取消</Button>
 *   <Button onClick={onConfirm}>确认</Button>
 * </ComicModalFooter>
 * ```
 */
export function ComicModalFooter({
  children,
  className = '',
}: Readonly<ComicModalFooterProps>) {
  return (
    <div className={`relative z-20 mt-6 flex gap-3 ${className}`}>
      {children}
    </div>
  );
}

ComicModalFooter.displayName = 'ComicModalFooter';

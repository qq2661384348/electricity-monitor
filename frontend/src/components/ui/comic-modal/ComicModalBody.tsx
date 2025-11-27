import { cn } from '@/lib/utils';
import type { ComicModalBodyProps } from './types';

/**
 * ComicModalBody - 模态框主体内容区域
 * 
 * 可滚动的内容容器
 * 
 * @example
 * ```tsx
 * <ComicModalBody>
 *   <p>这里是主要内容</p>
 * </ComicModalBody>
 * ```
 */
export function ComicModalBody({
  children,
  className = '',
  overflowVisible = false,
}: Readonly<ComicModalBodyProps>) {
  return (
    <div
      className={cn(
        "relative z-10 flex-1 min-h-0 pr-1",
        overflowVisible ? "overflow-visible" : "overflow-y-auto",
        className
      )}
    >
      {children}
    </div>
  );
}

ComicModalBody.displayName = 'ComicModalBody';

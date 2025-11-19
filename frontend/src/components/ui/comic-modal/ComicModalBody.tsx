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
}: ComicModalBodyProps) {
  return (
    <div className={`relative z-10 flex-1 overflow-y-auto pr-1 ${className}`}>
      {children}
    </div>
  );
}

ComicModalBody.displayName = 'ComicModalBody';

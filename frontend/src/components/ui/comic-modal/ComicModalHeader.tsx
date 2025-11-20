import { TITLE_TEXT_SHADOW } from './constants';
import type { ComicModalHeaderProps } from './types';

/**
 * ComicModalHeader - 模态框头部组件
 * 
 * 用于显示标题
 * 
 * @example
 * ```tsx
 * <ComicModalHeader>
 *   <h2>标题</h2>
 * </ComicModalHeader>
 * ```
 */
export function ComicModalHeader({
  children,
  className = '',
}: Readonly<ComicModalHeaderProps>) {
  return (
    <div className={`relative z-20 mb-6 ${className}`}>
      <div
        className="text-3xl font-black uppercase italic text-black"
        style={{ fontFamily: '"Bangers", cursive', textShadow: TITLE_TEXT_SHADOW }}
      >
        {children}
      </div>
    </div>
  );
}

ComicModalHeader.displayName = 'ComicModalHeader';

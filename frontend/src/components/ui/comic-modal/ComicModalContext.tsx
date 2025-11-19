import { createContext, useContext } from 'react';
import type { ComicModalContextValue } from './types';

/**
 * ComicModal Context
 * 用于在父子组件间共享模态框状态
 */
export const ComicModalContext = createContext<ComicModalContextValue | null>(null);

/**
 * 使用 ComicModal Context 的 Hook
 * @throws 如果在 ComicModalRoot 外部使用会抛出错误
 */
export function useComicModalContext(): ComicModalContextValue {
  const context = useContext(ComicModalContext);
  
  if (!context) {
    throw new Error(
      'useComicModalContext must be used within a ComicModalRoot. ' +
      'Wrap your component with <ComicModalRoot> or use the <ComicModal> wrapper.'
    );
  }
  
  return context;
}

import type { PropsWithChildren, ReactElement } from 'react';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, renderHook, type RenderOptions, type RenderHookOptions } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

function createWrapper(route = '/') {
  const queryClient = createTestQueryClient();

  function Wrapper({ children }: PropsWithChildren) {
    return (
      <MemoryRouter initialEntries={[route]}>
        <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
      </MemoryRouter>
    );
  }

  return { queryClient, Wrapper };
}

export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'> & { route?: string },
) {
  const { queryClient, Wrapper } = createWrapper(options?.route);

  return {
    queryClient,
    ...render(ui, {
      ...options,
      wrapper: Wrapper,
    }),
  };
}

export function renderHookWithProviders<Result, Props>(
  renderHookCallback: (initialProps: Props) => Result,
  options?: Omit<RenderHookOptions<Props>, 'wrapper'> & { route?: string },
) {
  const { queryClient, Wrapper } = createWrapper(options?.route);

  return {
    queryClient,
    ...renderHook(renderHookCallback, {
      ...options,
      wrapper: Wrapper,
    }),
  };
}

import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { renderWithProviders } from '@/test/render';

import { HeroDashboard } from './HeroDashboard';

describe('HeroDashboard', () => {
  it('renders the title without the extra translated yellow backdrop', () => {
    const { container } = renderWithProviders(<HeroDashboard />);

    expect(
      screen.getByRole('heading', { name: '🎓 面向校园宿舍场景的电费提醒系统' }),
    ).toBeInTheDocument();
    expect(
      container.querySelector('[class*="translate-x-2"][class*="translate-y-2"]'),
    ).not.toBeInTheDocument();
  });
});

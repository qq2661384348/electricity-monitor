import { http, HttpResponse } from 'msw';
import { act, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { server } from '@/test/msw/server';
import { renderHookWithProviders } from '@/test/render';

import { useAuthLoginFlow } from './useAuthLoginFlow';

async function waitForPublicConfig(result: { current: ReturnType<typeof useAuthLoginFlow> }) {
  await waitFor(() => {
    expect(result.current.emailLoginAvailable).toBe(true);
  });
}

describe('useAuthLoginFlow', () => {
  it('maps QQ USER_NOT_FRIEND verification-code errors with configured contacts', async () => {
    server.use(
      http.post('*/api/auth/send-verification-code', () =>
        HttpResponse.json({ error: 'USER_NOT_FRIEND' }, { status: 400 }),
      ),
    );

    const { result } = renderHookWithProviders(() => useAuthLoginFlow());

    await waitForPublicConfig(result);

    act(() => {
      result.current.updateIdentifier('123456789');
    });

    await act(async () => {
      await result.current.handleCaptchaSuccess('captcha-token');
    });

    expect(result.current.error).toBe(
      '请先添加机器人QQ号：3776431946。遇到问题请联系管理员：2661384348。',
    );
  });

  it('maps email verification-code error detail and normalizes the identifier', async () => {
    server.use(
      http.post('*/api/auth/send-verification-code', async ({ request }) => {
        const body = (await request.json()) as {
          login_mode?: string;
          identifier?: string;
          email?: string;
        };

        expect(body.login_mode).toBe('email');
        expect(body.identifier).toBe('student@example.com');
        expect(body.email).toBe('student@example.com');

        return HttpResponse.json({ detail: '邮箱验证码发送太频繁' }, { status: 429 });
      }),
    );

    const { result } = renderHookWithProviders(() => useAuthLoginFlow());

    await waitForPublicConfig(result);

    act(() => {
      result.current.switchLoginMode('email');
    });

    await waitFor(() => {
      expect(result.current.loginMode).toBe('email');
    });

    act(() => {
      result.current.updateIdentifier('Student@Example.COM');
    });

    await act(async () => {
      await result.current.handleCaptchaSuccess('captcha-token');
    });

    expect(result.current.error).toBe('邮箱验证码发送太频繁');
  });

  it('shows captcha token rejection detail when sending the verification code fails', async () => {
    const captchaTokens: string[] = [];
    server.use(
      http.post('*/api/auth/send-verification-code', async ({ request }) => {
        const body = (await request.json()) as { captcha_token?: string };
        captchaTokens.push(body.captcha_token ?? '');

        return HttpResponse.json({ detail: '图形验证码已过期' }, { status: 400 });
      }),
    );

    const { result } = renderHookWithProviders(() => useAuthLoginFlow());

    await waitForPublicConfig(result);

    act(() => {
      result.current.updateIdentifier('123456789');
    });

    await act(async () => {
      await result.current.handleCaptchaSuccess('expired-captcha-token');
    });

    expect(captchaTokens).toEqual(['expired-captcha-token']);
    expect(result.current.showCaptcha).toBe(false);
    expect(result.current.error).toBe('图形验证码已过期');
  });
});

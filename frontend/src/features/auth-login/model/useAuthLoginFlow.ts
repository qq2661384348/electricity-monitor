import { AxiosError } from 'axios';
import { useCallback, useEffect, useState, type KeyboardEvent } from 'react';

import { fallbackPublicConfig, type PublicConfig } from '@/entities/public-config';
import { usePublicConfig } from '@/features/public-config';
import { useAuthStore } from '@/stores/authStore';
import type { LoginMode } from '@/types';

import { authApi } from '../api/authApi';
import {
  isValidLoginIdentifier,
  loginModeLabel,
  normalizeLoginIdentifier,
  sanitizeLoginIdentifier,
} from './loginIdentity';

const SEND_CODE_FALLBACK_MESSAGE = '发送失败，请稍后重试';
const LOGIN_FALLBACK_MESSAGE = '验证失败，请检查验证码';
const RESEND_COUNTDOWN_SECONDS = 60;

type ApiErrorData = Record<string, unknown>;

function getApiErrorData(err: unknown): ApiErrorData | null {
  if (!(err instanceof AxiosError)) {
    return null;
  }

  const data = err.response?.data;
  if (!data || typeof data !== 'object') {
    return null;
  }

  return data as ApiErrorData;
}

function readApiString(data: ApiErrorData, key: string): string | null {
  const value = data[key];
  if (typeof value !== 'string') {
    return null;
  }

  const trimmed = value.trim();
  return trimmed || null;
}

export function formatUserNotFriendMessage(publicConfig: PublicConfig): string {
  const botQQ = publicConfig.notification.qq_bot_public_qq_number.trim();
  const adminQQ = publicConfig.notification.admin_qq_number.trim();
  const botText = botQQ || '请联系管理员获取机器人QQ号';
  const adminText = adminQQ || '请联系管理员';
  return `请先添加机器人QQ号：${botText}。遇到问题请联系管理员：${adminText}。`;
}

export function getSendVerificationCodeErrorMessage(
  err: unknown,
  publicConfig: PublicConfig,
): string {
  const data = getApiErrorData(err);
  if (!data) {
    return SEND_CODE_FALLBACK_MESSAGE;
  }

  if (data.error === 'USER_NOT_FRIEND') {
    return formatUserNotFriendMessage(publicConfig);
  }

  return (
    readApiString(data, 'detail') ??
    readApiString(data, 'message') ??
    SEND_CODE_FALLBACK_MESSAGE
  );
}

export function getLoginErrorMessage(err: unknown): string {
  const data = getApiErrorData(err);
  if (!data) {
    return LOGIN_FALLBACK_MESSAGE;
  }

  return (
    readApiString(data, 'detail') ??
    readApiString(data, 'message') ??
    LOGIN_FALLBACK_MESSAGE
  );
}

interface UseAuthLoginFlowOptions {
  readonly onLoginSuccess?: () => void;
}

export function useAuthLoginFlow(options: UseAuthLoginFlowOptions = {}) {
  const { onLoginSuccess } = options;
  const { data: publicConfig } = usePublicConfig();
  const resolvedPublicConfig = publicConfig ?? fallbackPublicConfig;
  const codeLength = resolvedPublicConfig.verification.code_length;
  const emailLoginAvailable =
    resolvedPublicConfig.auth.email_login_enabled &&
    resolvedPublicConfig.auth.login_modes.includes('email');
  const login = useAuthStore((state) => state.login);

  const [loginMode, setLoginMode] = useState<LoginMode>('qq');
  const [qqNumber, setQqNumber] = useState('');
  const [email, setEmail] = useState('');
  const [code, setCode] = useState('');
  const [countdown, setCountdown] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCaptcha, setShowCaptcha] = useState(false);

  useEffect(() => {
    if (countdown <= 0) {
      return undefined;
    }

    const timer = setTimeout(() => setCountdown((current) => current - 1), 1000);
    return () => clearTimeout(timer);
  }, [countdown]);

  const currentIdentifier = loginMode === 'qq' ? qqNumber : email;
  const currentIdentifierLabel = loginModeLabel(loginMode);
  const normalizedIdentifier = normalizeLoginIdentifier(loginMode, currentIdentifier);
  const identifierValid =
    loginMode === 'email' && !emailLoginAvailable
      ? false
      : isValidLoginIdentifier(loginMode, currentIdentifier);

  const switchLoginMode = useCallback(
    (mode: LoginMode) => {
      if (mode === 'email' && !emailLoginAvailable) {
        setError('邮箱登录暂未启用');
        return;
      }

      setLoginMode(mode);
      setCode('');
      setError(null);
    },
    [emailLoginAvailable],
  );

  const updateIdentifier = useCallback(
    (value: string) => {
      const sanitized = sanitizeLoginIdentifier(loginMode, value);
      if (loginMode === 'qq') {
        setQqNumber(sanitized);
      } else {
        setEmail(sanitized);
      }
    },
    [loginMode],
  );

  const updateCode = useCallback(
    (value: string) => {
      setCode(value.replaceAll(/\D/g, '').slice(0, codeLength));
    },
    [codeLength],
  );

  const requestVerificationCode = useCallback(() => {
    if (!identifierValid) {
      setError(`请输入有效的${currentIdentifierLabel}`);
      return;
    }

    setShowCaptcha(true);
  }, [currentIdentifierLabel, identifierValid]);

  const closeCaptcha = useCallback(() => {
    setShowCaptcha(false);
  }, []);

  const handleCaptchaSuccess = useCallback(
    async (token: string) => {
      setShowCaptcha(false);
      setIsLoading(true);
      setError(null);

      try {
        await authApi.sendVerificationCode(loginMode, normalizedIdentifier, token);
        setCountdown(RESEND_COUNTDOWN_SECONDS);
      } catch (err: unknown) {
        setError(getSendVerificationCodeErrorMessage(err, resolvedPublicConfig));
      } finally {
        setIsLoading(false);
      }
    },
    [loginMode, normalizedIdentifier, resolvedPublicConfig],
  );

  const submitLogin = useCallback(async () => {
    if (!identifierValid || code.length !== codeLength) {
      setError(`请输入有效的${currentIdentifierLabel}和${codeLength}位验证码`);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await authApi.verifyAndLogin(loginMode, normalizedIdentifier, code);
      login(response.access_token, response.user);
      onLoginSuccess?.();
    } catch (err: unknown) {
      setError(getLoginErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [
    code,
    codeLength,
    currentIdentifierLabel,
    identifierValid,
    login,
    loginMode,
    normalizedIdentifier,
    onLoginSuccess,
  ]);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key !== 'Enter' || isLoading) {
        return;
      }

      if (code.length === codeLength) {
        void submitLogin();
      } else if (identifierValid && countdown === 0) {
        requestVerificationCode();
      }
    },
    [code, codeLength, countdown, identifierValid, isLoading, requestVerificationCode, submitLogin],
  );

  return {
    code,
    codeLength,
    codePlaceholder: `${codeLength}位验证码`,
    countdown,
    currentIdentifier,
    currentIdentifierLabel,
    emailLoginAvailable,
    error,
    handleCaptchaSuccess,
    handleKeyDown,
    identifierValid,
    isLoading,
    loginMode,
    publicConfig: resolvedPublicConfig,
    requestVerificationCode,
    closeCaptcha,
    showCaptcha,
    submitLogin,
    switchLoginMode,
    updateCode,
    updateIdentifier,
  };
}

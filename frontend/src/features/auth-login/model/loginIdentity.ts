import type { LoginMode, User } from '@/types';

const EMAIL_PATTERN = /^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,254}$/;

export function sanitizeLoginIdentifier(mode: LoginMode, value: string): string {
  if (mode === 'qq') {
    return value.replaceAll(/\D/g, '');
  }

  return value.trim();
}

export function normalizeLoginIdentifier(mode: LoginMode, value: string): string {
  const sanitized = sanitizeLoginIdentifier(mode, value);
  return mode === 'email' ? sanitized.toLowerCase() : sanitized;
}

export function isValidLoginIdentifier(mode: LoginMode, value: string): boolean {
  const normalized = normalizeLoginIdentifier(mode, value);
  if (mode === 'qq') {
    return /^\d{5,20}$/.test(normalized);
  }

  return EMAIL_PATTERN.test(normalized);
}

export function loginModeLabel(mode: LoginMode): string {
  return mode === 'qq' ? 'QQ号码' : '邮箱地址';
}

export function getUserDisplayIdentifier(user: User): string {
  return user.identifier || user.email || user.qq_number || '未知账号';
}

//! 验证码邮件模板

use super::error::{EmailError, Result};

/// 验证码邮件渲染结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedEmail {
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

/// 验证码邮件场景
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationScene {
    Register,
    Login,
    Reset,
    Bind,
    Unbind,
}

impl VerificationScene {
    fn title(self) -> &'static str {
        match self {
            VerificationScene::Register => "注册验证码",
            VerificationScene::Login => "登录验证码",
            VerificationScene::Reset => "密码重置验证码",
            VerificationScene::Bind => "邮箱绑定验证码",
            VerificationScene::Unbind => "邮箱解绑验证码",
        }
    }

    fn headline(self) -> &'static str {
        match self {
            VerificationScene::Register => "账户注册验证",
            VerificationScene::Login => "账户登录验证",
            VerificationScene::Reset => "密码重置验证",
            VerificationScene::Bind => "邮箱绑定验证",
            VerificationScene::Unbind => "邮箱解绑验证",
        }
    }

    fn description(self) -> &'static str {
        match self {
            VerificationScene::Register => {
                "感谢您注册账户。为了确保账户安全，请使用以下验证码完成注册。"
            }
            VerificationScene::Login => {
                "您正在尝试登录账户。为了确认这是本人操作，请使用以下验证码完成登录。"
            }
            VerificationScene::Reset => {
                "您正在申请重置账户密码。为了保护账户安全，请使用以下验证码完成密码重置。"
            }
            VerificationScene::Bind => {
                "您正在将此邮箱绑定到账户。为了确认邮箱所有权，请使用以下验证码完成绑定。"
            }
            VerificationScene::Unbind => {
                "您正在申请解绑此邮箱。为了确认您的身份，请使用以下验证码完成解绑。"
            }
        }
    }

    fn valid_minutes(self) -> u32 {
        match self {
            VerificationScene::Register | VerificationScene::Login => 10,
            VerificationScene::Reset => 15,
            VerificationScene::Bind | VerificationScene::Unbind => 30,
        }
    }

    fn accent_color(self) -> &'static str {
        match self {
            VerificationScene::Register => "#2e7d32",
            VerificationScene::Login => "#1565c0",
            VerificationScene::Reset => "#ef6c00",
            VerificationScene::Bind => "#6a1b9a",
            VerificationScene::Unbind => "#c62828",
        }
    }

    fn warning(self) -> &'static str {
        match self {
            VerificationScene::Register => {
                "如果您没有进行注册操作，请忽略此邮件。请不要将验证码告诉任何人。"
            }
            VerificationScene::Login => {
                "如果这不是您本人的登录操作，请立即检查账户安全。请不要将验证码告诉任何人。"
            }
            VerificationScene::Reset => {
                "如果您没有申请密码重置，可能有人正在尝试访问您的账户，请立即检查账户安全。"
            }
            VerificationScene::Bind => {
                "如果您没有进行邮箱绑定操作，请忽略此邮件。绑定成功后，此邮箱可能用于接收重要通知。"
            }
            VerificationScene::Unbind => {
                "解绑邮箱可能影响安全通知和找回能力。如果您没有进行解绑操作，请立即检查账户安全。"
            }
        }
    }
}

impl TryFrom<&str> for VerificationScene {
    type Error = EmailError;

    fn try_from(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "register" => Ok(VerificationScene::Register),
            "login" => Ok(VerificationScene::Login),
            "reset" | "reset_password" => Ok(VerificationScene::Reset),
            "bind" => Ok(VerificationScene::Bind),
            "unbind" => Ok(VerificationScene::Unbind),
            other => Err(EmailError::Validation(format!(
                "无效的验证码邮件场景: {other}"
            ))),
        }
    }
}

/// 渲染验证码邮件。
pub fn render_verification_code(
    code: &str,
    scene: VerificationScene,
    from_name: &str,
) -> Result<RenderedEmail> {
    let code = code.trim();
    if code.is_empty() || !code.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(EmailError::Validation(
            "验证码必须是非空数字字符串".to_string(),
        ));
    }

    let app_name = if from_name.trim().is_empty() {
        "Electricity Monitor"
    } else {
        from_name.trim()
    };
    let escaped_app_name = escape_html(app_name);
    let escaped_code = escape_html(code);
    let subject = format!("【{app_name}】{}", scene.title());
    let valid_minutes = scene.valid_minutes();

    let text_body = format!(
        "{app_name} - {headline}\n\
         ================================\n\n\
         您好！\n\n\
         {description}\n\n\
         验证码：{code}\n\n\
         使用说明：\n\
         - 请在页面中输入上述验证码\n\
         - 验证码有效期为 {valid_minutes} 分钟\n\
         - 验证码仅可使用一次\n\n\
         安全提醒：\n\
         {warning}\n\n\
         此邮件由系统自动发送，请勿回复。",
        headline = scene.headline(),
        description = scene.description(),
        warning = scene.warning(),
    );

    let html_body = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
</head>
<body style="margin:0;padding:24px;background:#f6f7f9;color:#222;font-family:Arial,'Microsoft YaHei',sans-serif;">
  <main style="max-width:600px;margin:0 auto;background:#fff;border-radius:8px;padding:28px;border:1px solid #e6e8eb;">
    <header style="border-bottom:2px solid {color};padding-bottom:18px;margin-bottom:24px;text-align:center;">
      <div style="font-size:24px;font-weight:700;color:{color};">{app_name}</div>
      <h1 style="font-size:20px;line-height:1.4;margin:8px 0 0;">{headline}</h1>
    </header>
    <p style="font-size:16px;line-height:1.7;margin:0 0 18px;">您好！</p>
    <p style="font-size:15px;line-height:1.8;margin:0 0 20px;">{description}</p>
    <section style="background:#f8f9fa;border:2px dashed {color};border-radius:8px;padding:20px;text-align:center;margin:22px 0;">
      <div style="font-size:14px;color:#666;margin-bottom:8px;">验证码</div>
      <div style="font-size:32px;font-weight:700;color:{color};letter-spacing:8px;font-family:'Courier New',monospace;">{code}</div>
    </section>
    <p style="font-size:14px;line-height:1.8;color:#555;margin:0 0 18px;">
      请在页面中输入上述验证码。验证码有效期为 {valid_minutes} 分钟，且仅可使用一次。
    </p>
    <aside style="background:#fff8e1;border:1px solid #ffe082;border-radius:6px;padding:14px;font-size:14px;line-height:1.7;color:#5d4037;">
      {warning}
    </aside>
    <footer style="font-size:12px;color:#888;border-top:1px solid #edf0f2;margin-top:24px;padding-top:16px;text-align:center;">
      此邮件由系统自动发送，请勿回复。
    </footer>
  </main>
</body>
</html>"#,
        title = scene.title(),
        color = scene.accent_color(),
        app_name = escaped_app_name,
        headline = scene.headline(),
        description = scene.description(),
        code = escaped_code,
        warning = scene.warning(),
    );

    Ok(RenderedEmail {
        subject,
        text_body,
        html_body,
    })
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_aliases_reset_password() {
        let scene = VerificationScene::try_from("reset_password").unwrap();
        assert_eq!(scene, VerificationScene::Reset);
    }

    #[test]
    fn test_rejects_unknown_scene() {
        let error = VerificationScene::try_from("unknown").unwrap_err();
        assert!(error.to_string().contains("无效的验证码邮件场景"));
    }

    #[test]
    fn test_render_verification_email_contains_code() {
        let rendered =
            render_verification_code("123456", VerificationScene::Login, "CogniAegis").unwrap();

        assert!(rendered.subject.contains("登录验证码"));
        assert!(rendered.text_body.contains("123456"));
        assert!(rendered.html_body.contains("123456"));
        assert!(rendered.html_body.contains("CogniAegis"));
    }

    #[test]
    fn test_rejects_non_digit_code() {
        let error =
            render_verification_code("abc123", VerificationScene::Login, "CogniAegis").unwrap_err();
        assert!(error.to_string().contains("验证码"));
    }
}

//! 验证码邮件模板

use super::error::{EmailError, Result};
use crate::domain::models::Room;

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

/// 渲染电费预警邮件。
///
/// QQ 机器人通知是纯文本，邮件通知在保留相同核心信息的基础上提供 HTML 版式，
/// 便于用户在邮箱客户端中快速识别房间、剩余电量、阈值和访问入口。
pub fn render_electricity_alert(
    room: &Room,
    public_url: &str,
    from_name: &str,
) -> Result<RenderedEmail> {
    let app_name = if from_name.trim().is_empty() {
        "Electricity Monitor"
    } else {
        from_name.trim()
    };
    let public_url = public_url.trim();
    if public_url.is_empty() {
        return Err(EmailError::Validation(
            "电费预警邮件需要公开访问地址".to_string(),
        ));
    }

    let escaped_app_name = escape_html(app_name);
    let escaped_room_path = escape_html(&room.primary_roompath);
    let escaped_public_url = escape_html(public_url);
    let subject = format!("【{app_name}】电量预警提醒");
    let current = format!("{:.2}", room.electricity_fee);
    let threshold = format!("{:.2}", room.threshold);

    let text_body = format!(
        "{app_name} - 电量预警提醒\n\
         ================================\n\n\
         房间位置：{room_path}\n\
         当前剩余：{current} kWh\n\
         预警阈值：{threshold} kWh\n\n\
         您的电量已低于预警阈值，请及时充值。\n\n\
         访问 {public_url} 以更新你的数据。\n\n\
         此邮件由系统自动发送，请勿回复。",
        room_path = room.primary_roompath,
    );

    let html_body = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>电量预警提醒</title>
</head>
<body style="margin:0;padding:24px;background:#f6f7f9;color:#111827;font-family:Arial,'Microsoft YaHei',sans-serif;">
  <main style="max-width:640px;margin:0 auto;background:#fff;border:1px solid #e5e7eb;border-radius:8px;overflow:hidden;">
    <header style="background:#111827;color:#fff;padding:22px 28px;">
      <div style="font-size:14px;letter-spacing:1px;text-transform:uppercase;color:#facc15;">{app_name}</div>
      <h1 style="font-size:24px;line-height:1.35;margin:8px 0 0;">电量预警提醒</h1>
    </header>
    <section style="padding:26px 28px;">
      <p style="font-size:15px;line-height:1.8;margin:0 0 18px;">您的电量已低于预警阈值，请及时充值。</p>
      <div style="border:2px solid #111827;border-radius:8px;overflow:hidden;margin:20px 0;">
        <div style="padding:14px 16px;background:#facc15;font-weight:700;">房间位置</div>
        <div style="padding:16px;font-size:16px;line-height:1.6;">{room_path}</div>
      </div>
      <table role="presentation" width="100%" cellspacing="0" cellpadding="0" style="border-collapse:collapse;margin:20px 0;">
        <tr>
          <td style="width:50%;padding:14px;border:1px solid #e5e7eb;background:#f9fafb;">
            <div style="font-size:13px;color:#6b7280;margin-bottom:6px;">当前剩余</div>
            <div style="font-size:26px;font-weight:800;color:#dc2626;">{current} kWh</div>
          </td>
          <td style="width:50%;padding:14px;border:1px solid #e5e7eb;background:#f9fafb;">
            <div style="font-size:13px;color:#6b7280;margin-bottom:6px;">预警阈值</div>
            <div style="font-size:26px;font-weight:800;color:#111827;">{threshold} kWh</div>
          </td>
        </tr>
      </table>
      <a href="{public_url}" style="display:inline-block;background:#111827;color:#fff;text-decoration:none;font-weight:700;padding:12px 18px;border-radius:6px;">访问系统更新数据</a>
      <p style="font-size:13px;line-height:1.7;color:#6b7280;margin:18px 0 0;">如果按钮无法打开，请复制此链接访问：{public_url}</p>
    </section>
    <footer style="font-size:12px;color:#6b7280;border-top:1px solid #e5e7eb;padding:16px 28px;text-align:center;">
      此邮件由系统自动发送，请勿回复。
    </footer>
  </main>
</body>
</html>"#,
        app_name = escaped_app_name,
        room_path = escaped_room_path,
        public_url = escaped_public_url,
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

    #[test]
    fn test_render_electricity_alert_email_contains_room_data() {
        let room = Room {
            id: uuid::Uuid::new_v4(),
            roomid: 101,
            electricity_fee: 5.5,
            send_flag: true,
            threshold: 10.0,
            room_name: "测试房间".to_string(),
            primary_roompath: "南校区/1号楼/101".to_string(),
            primary_roompath_hash: 12345678,
            has_additional_paths: false,
            is_active: true,
            source_type: "manual".to_string(),
            external_id: None,
            last_synced_at: None,
            last_recovered_at: None,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        let rendered =
            render_electricity_alert(&room, "https://example.com", "CogniAegis").unwrap();

        assert!(rendered.subject.contains("电量预警"));
        assert!(rendered.text_body.contains("南校区/1号楼/101"));
        assert!(rendered.html_body.contains("5.50 kWh"));
        assert!(rendered.html_body.contains("10.00 kWh"));
        assert!(rendered.html_body.contains("https://example.com"));
        assert!(!rendered.text_body.contains("roomid"));
    }
}

# Traceability

| Baseline / Risk / Unknown | Requirement | Spec | Design | Task | Assertion | Scorecard | 状态 | 备注 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B-002 / RISK-001 | REQ-001 | SPEC-email-config | D-002 / D-006 | TASK-001 / TASK-004 | ACC-001 / ACC-006 | SC-security | active | password 不落 tracked 文件 |
| B-004 | REQ-002 | SPEC-email-sender | D-003 / D-004 | TASK-002 | ACC-002 / ACC-003 | SC-functional | active | 参考 SMTP 发送能力 |
| B-004 | REQ-003 | SPEC-email-templates | D-005 | TASK-003 | ACC-004 / ACC-005 | SC-functional | active | 验证码场景 |
| B-003 / RISK-004 | REQ-004 | SPEC-source-sync | D-006 | TASK-004 / TASK-005 | ACC-006 / ACC-007 | SC-source-sync | active | release secret 同步 |
| U-001 | REQ-005 | SPEC-nongoal | D-001 | TASK-006 | ACC-008 | SC-minimality | active | 不接入 auth/API |

```yaml
traceability:
  - baseline: B-002/RISK-001
    requirement: REQ-001
    spec: SPEC-email-config
    design: D-002,D-006
    task: TASK-001,TASK-004
    assertion: ACC-001,ACC-006
    scorecard: SC-security
    status: active
    notes: "password must stay out of tracked files"
  - baseline: B-004
    requirement: REQ-002
    spec: SPEC-email-sender
    design: D-003,D-004
    task: TASK-002
    assertion: ACC-002,ACC-003
    scorecard: SC-functional
    status: active
    notes: "SMTP sender"
  - baseline: B-004
    requirement: REQ-003
    spec: SPEC-email-templates
    design: D-005
    task: TASK-003
    assertion: ACC-004,ACC-005
    scorecard: SC-functional
    status: active
    notes: "verification templates"
  - baseline: B-003/RISK-004
    requirement: REQ-004
    spec: SPEC-source-sync
    design: D-006
    task: TASK-004,TASK-005
    assertion: ACC-006,ACC-007
    scorecard: SC-source-sync
    status: active
    notes: "release secret"
  - baseline: U-001
    requirement: REQ-005
    spec: SPEC-nongoal
    design: D-001
    task: TASK-006
    assertion: ACC-008
    scorecard: SC-minimality
    status: active
    notes: "no auth/api integration"
```

---
name: tarot
description: |
  Use when the user asks for tarot/塔罗/占卜/抽牌/牌阵/每日一牌/运势/感情、事业、决策指引. Use the native Rust tarot CLI for reproducible draws, Chinese upright/reversed card names, agency-first interpretation and safety boundaries.
---

# Tarot 塔罗占卜（Rust-native）

塔罗是镜子，不是水晶球：把牌面转成自我观察和可选择的下一步，不宣判固定命运。

## 必守规则

- 模糊问题先确认主题；若已有背景锚点，解读必须回扣具体行为。
- **必须先调用原生 Rust CLI 抽牌**，不凭感觉选牌。输出需要展示 `seed` 和 `time_factor`。
- 主权归用户：不说“你一定会”；结尾提醒“牌显示的是当下能量，你的选择随时可以改变走向”。
- 反巴纳姆：建议具体到时间、动作，不堆“注意沟通、保持平衡、相信直觉、一切都会好”。
- 不把牌用作医疗、法律、投资买卖或重大人生决定的依据。明确自伤风险时暂停占卜，优先鼓励联系当地紧急服务或可信任的人。

## 抽牌：原生 Rust CLI

本仓库执行 `cargo build --release`，或 `cargo install --path .` 安装 `tarot`。不需要 Python、网络或 UI。

```sh
tarot draw --spread three --question "事业方向" --json
tarot draw --spread three --seed 123 --time-factor night --question "事业方向" --json
tarot analyze --spread three --seed 123 --time-factor night --json
tarot deck list --json
tarot spreads list --json
```

如未安装，可在本仓库使用 `cargo run -- draw --spread three --question "事业方向"`。

**复现:** 必须同时传入 `--seed` 和 `--time-factor`。Python 旧版与 Rust 新版使用不同的伪随机算法，因此旧种子不会产出相同顺序。

推荐：`single` 今日/简单问题；`three` 通用；`diamond` 决策；`moon` 月度；`horseshoe` 时间线；`celtic` 复杂局面。位置见 `references/spreads.md`。

## 解读

逐牌选 1–2 个透镜：镜子=当前状态；窗户=盲点/潜意识；门=行动；锚=固定信念。牌义见 `references/cards.md`。

多牌必须考虑：
- 花色/元素：3+ 同花色为主导；缺席花色仅是可以检视的维度。
- 大阿卡纳：大于 50% 可读为重大转折，30–50% 命运与选择并存，低于 30% 日常主动选择占主导。
- 相邻牌关系：因果/对话/递进/转折，附上 A1/A2/A3/B1/B2/B3/经典组合等来源；可由 `tarot analyze` 获取**启发式提示**。不是唯一正确的解释。参见 `references/card-relations.md` 和 `references/combinations.md`。

综合解读：起点 → 张力 → 转折 → 出口 → 回响。“出口”必须是本周具体可执行的一件小事。

## 输出

中文优先，温暖但清醒。给出整体能量、每张牌位置和正逆位（附 seed/time_factor）、逐牌透镜、牌间关系、综合行动建议、3–4 字总结词，并以开放问题结尾，把决定权还给用户。

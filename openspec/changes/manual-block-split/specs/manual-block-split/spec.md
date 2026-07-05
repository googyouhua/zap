## ADDED Requirements

### Requirement: 手动分割块

系统在终端中提供一个快捷键，当用户按下时，根据当前终端光标位置分割 active block。

#### Scenario: 在输出行中分割

- **GIVEN** shell 集成未启用，终端显示块模式
- **AND** active block 的 output 包含多行内容
- **WHEN** 用户将光标定位在某输出行并按下 Ctrl+Shift+\
- **THEN** 光标行之前的输出保留在原 block
- **AND** 光标行文本作为新 block 的 command
- **AND** 光标行之后的输出作为新 block 的 output

#### Scenario: 在最后一行分割

- **GIVEN** 光标在 active block output 的最后一行
- **AND** 该行为空行
- **WHEN** 用户按下 Ctrl+Shift+\
- **THEN** 不执行分割（空 command 无意义）

#### Scenario: 在首行分割

- **GIVEN** 光标在 active block output 的第一行
- **WHEN** 用户按下 Ctrl+Shift+\
- **THEN** 第一行文本提取为新 block 的 command
- **AND** 其余行作为新 block 的 output
- **AND** 原 block output 为空

#### Scenario: 快捷键上下文

- **WHEN** 终端失焦或 IME 打开
- **THEN** 快捷键不生效

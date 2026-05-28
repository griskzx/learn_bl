# STM32 Bootloader 学习进度记录

> 目标平台：STM32F439（Cortex-M4F）  
> 开发语言：Rust（裸机 `#![no_std]`）  
> 最后更新：2026-05-28

---

## 项目结构

```
learn_bl/
├── bootloader/          # Bootloader 工程（烧录到 Flash 起始 0x0800_0000）
│   └── src/
│       ├── main.rs      # 入口：校验固件头，决定是否跳转
│       ├── bl.rs        # 核心跳转逻辑（jump_to_app）
│       ├── config.rs    # 时钟初始化
│       └── firmware.rs  # FirmwareHeader 结构体定义（供 bootloader 读取）
├── app/                 # 应用固件工程（烧录到 0x0801_0000）
│   └── src/
│       ├── main.rs      # 业务逻辑：PB0 LED 以 500ms 闪烁
│       ├── lib.rs       # panic handler / HardFault handler
│       └── firmware.rs  # 固件头静态变量（放入 .fw_header 段）
├── patch_header.py      # 编译后修补 bin 文件的 size 和 CRC32
└── PROGRESS.md          # 本文件
```

---

## 阶段一：项目初始化 ✅

**提交：** `Initial commit: STM32 bootloader project`

- 搭建 Cargo workspace，分为 `bootloader` 和 `app` 两个独立 crate
- 配置各自的 `memory.x` linker script，分别将 Flash 起始地址设为：
  - Bootloader：`0x0800_0000`
  - App：`0x0801_0000`（偏移 64 KB）
- 引入依赖：`stm32f4xx-hal`、`cortex-m`、`cortex-m-rt`、`defmt`、`panic-probe`
- 配置 `Embed.toml`（probe-rs 调试）和 `.cargo/config.toml`（编译目标 thumbv7em-none-eabihf）

---

## 阶段二：固件头 + 跳转机制 ✅

**提交：** `feat: firmware header validation and jump-to-app`

### 2.1 固件头（FirmwareHeader）

定义了一个放在 Flash 中固定位置的 C 兼容结构体：

```rust
#[repr(C)]
pub struct FirmwareHeader {
    pub magic:   u32,   // 魔数，用于验证固件有效性
    pub version: u32,   // 语义化版本号（BCD 格式）
    pub size:    u32,   // 代码段实际字节数（由 patch_header.py 注入）
    pub crc:     u32,   // CRC32 校验和（由 patch_header.py 注入）
}
```

App 侧通过链接器段 `.fw_header` 将其放到 Flash 的最开头（`0x0801_0000`）：

```rust
#[unsafe(link_section = ".fw_header")]
#[unsafe(no_mangle)]
#[used]
pub static FW_HEADER: FirmwareHeader = FirmwareHeader {
    magic:   0xA9B3ED,
    version: 0x0001_0000,
    size:    0xFFFFFFFF,  // 编译后由 patch_header.py 填入
    crc:     0xFFFFFFFF,  // 编译后由 patch_header.py 填入
};
```

### 2.2 Bootloader 启动流程

```
上电复位
  │
  ▼
初始化时钟（HSE 8MHz → PLL → 168MHz）
  │
  ▼
从 0x0801_0000 读取 FirmwareHeader（volatile 读防止优化）
  │
  ├─ magic == 0xA9B3ED ──► jump_to_app(0x0801_0000)
  │
  └─ 否则 ──► 点亮 PB14 红灯，死循环等待重新烧录
```

### 2.3 jump_to_app 实现要点（bl.rs）

跳转前需要完整还原硬件状态，否则 App 的初始化代码会出现竞态。当前实现步骤：

| 步骤 | 操作 | 原因 |
|------|------|------|
| 1 | 停止 SysTick 计数器 & 中断 | 避免跳转后 SysTick 仍触发 Bootloader 的 handler |
| 2 | 清零 NVIC_ICER / NVIC_ICPR（全部 8 组） | 禁用并清除所有挂起的外部中断 |
| 3 | 复位 RCC 到出厂状态（切回 HSI，关闭 PLL/HSE） | 防止 App 时钟初始化时 PLL 已锁定造成超时 |
| 4 | 复位 AHB1/AHB2/APB1/APB2 所有外设 | 清除 Bootloader 残留的 GPIO/UART 等状态 |
| 5 | 写入 `SCB.VTOR` = App 基地址 | 使 Cortex-M 异常向量表指向 App |
| 6 | DSB + ISB 内存屏障 | 确保流水线和缓存一致性 |
| 7 | 读取 App 向量表：SP（偏移 0）& Reset_Handler（偏移 4） | ARM Cortex-M 启动规范 |
| 8 | `msr msp, sp` + `bx reset_handler`（内联 ASM） | 设置主栈指针并跳转，不再返回 |

> **注意**：未关闭全局中断（PRIMASK），保留 App 启动后正常响应中断的能力。

### 2.4 CRC32 修补脚本（patch_header.py）

编译完成后运行：

```bash
python patch_header.py app/app.bin
```

脚本行为：
1. 跳过 bin 文件前 256 字节（固件头占位区）
2. 计算剩余代码的 `zlib.crc32`
3. 将 `size` 和 `crc32` 用 little-endian 写回 bin 偏移 8 和 12 字节处

---

## 待完成 / 下一步计划 🚧

- [ ] **在 Bootloader 中实现 CRC32 验证**：读取 `header.size` 字节，计算 CRC 并与 `header.crc` 比对，失败时拒绝跳转
- [ ] **UART/USB DFU 升级**：通过串口接收新固件并写入 Flash（需要 Flash 擦写驱动）
- [ ] **双 Bank 回滚**：保留旧固件，新固件验证失败时自动回退
- [ ] **版本号比对**：防止降级攻击

---

## 常用命令

```bash
# 编译并烧录 Bootloader
cd bootloader && cargo embed --release

# 编译 App，生成 bin，修补 header，再烧录
cd app
cargo objcopy --release -- -O binary app.bin
python ../patch_header.py app.bin
probe-rs download --binary-format bin --base-address 0x08010000 app.bin --chip STM32F439ZITx
```

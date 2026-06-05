/// 使用 STM32F439 的硬件 CRC 外设计算 32 位校验和。
///
/// 由于 STM32F4 的 CRC 外设硬件上只支持以 32 位字（Word）为单位进行写入，
/// 输入数据 `data` 必须是 `&[u32]`。
///
/// 硬件 CRC 结果与标准 IEEE CRC-32 (如 Python `zlib.crc32`) 的唯一区别是
/// 硬件没有在最后进行异或 `0xFFFFFFFF`，因此我们读出结果后需要手动进行异或。
pub fn hw_crc32(data: &[u32]) -> u32 {
    let dp = unsafe { stm32f4xx_hal::pac::Peripherals::steal() };
    let crc = dp.CRC;

    // 1. 复位 CRC 外设（清空数据寄存器，初始值设为 0xFFFFFFFF）
    crc.cr().write(|w| w.reset().reset());

    // 2. 逐字写入数据寄存器进行计算
    for word in data {
        crc.dr().write(|w| unsafe { w.dr().bits(*word) });
    }

    // 3. 读取结果并进行最终异或，以适配 zlib/IEEE CRC-32 校验和
    let result = crc.dr().read().dr().bits();
    result ^ 0xFFFF_FFFF
}

use stm32f4xx_hal::pac::FLASH;

const FLASH_KEY1:u32 = 0x45670123;
const FLASH_KEY2:u32 = 0xCDEF89AB;

unsafe fn flash_unlock(){
    let flash = unsafe { &*FLASH::ptr() };
    if flash.cr().read().lock().bit_is_set(){
        flash.keyr().write(|w| unsafe {
            w.bits(FLASH_KEY1)
        });
        flash.keyr().write(|w| unsafe { w.bits(FLASH_KEY2) });
    }
}
unsafe fn flash_lock(){
    let flash = unsafe { &*FLASH::ptr() };
    flash.cr().modify(|_,w| w.lock().set_bit());
}
unsafe fn wait_flash_ready() {
       let flash = unsafe { &*FLASH::ptr() };
       // 忙等 BSY (Busy) 位清除
       while flash.sr().read().bsy().bit_is_set() {}
}
pub unsafe fn flash_write_words(mut addr: u32, data: &[u32]) -> Result<(),()> {
       let flash = unsafe { &*FLASH::ptr() };

       unsafe {
           flash_unlock();
           wait_flash_ready();
       }

       // 1. 设置写模式为 32 位并行写入（PSIZE = 0b10）
       flash.cr().modify(|_, w| unsafe { w.psize().bits(0b10) });
       // 2. 开启编程使能 (PG = 1)
       flash.cr().modify(|_, w| w.pg().set_bit());

       for &word in data {
           // 3. 往 Flash 物理地址直接写入字
           unsafe {
               core::ptr::write_volatile(addr as *mut u32, word);
               wait_flash_ready();
           }

           // 4. 简单校验是否写入成功
           let success = unsafe { core::ptr::read_volatile(addr as *const u32) == word };
           if !success {
               // 写入失败，清除编程使能并加锁
               flash.cr().modify(|_, w| w.pg().clear_bit());
               unsafe { flash_lock(); }
               return Err(());
           }
           addr += 4;
       }

       // 5. 写入结束，清除编程使能并加锁
       flash.cr().modify(|_, w| w.pg().clear_bit());
       unsafe { flash_lock(); }
       Ok(())
   }
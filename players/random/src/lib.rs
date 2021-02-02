extern "C" {
    fn guess(n: u32) -> i8;
    fn rand32(a: u32, b: u32) -> u32;
}

static mut LOWER: u32 = 0;
static mut UPPER: u32 = std::u32::MAX;

#[no_mangle]
pub extern "C" fn turn() {
    let r = unsafe {rand32(LOWER, UPPER)};
    unsafe{
        let res = guess(r);
        if res > 0 {
            UPPER = r - 1;
        }
        if res < 0 {
            LOWER = r + 1;
        }
    }
}

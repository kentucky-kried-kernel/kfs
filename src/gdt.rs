#[no_mangle]
static GDT_LIMIT: usize = 3;
#[no_mangle]
static GDT: [u64; GDT_LIMIT] = [0x0, KERNEL_CODE_SEG, KERNEL_DATA_SEG];

const KERNEL_CODE_SEG: u64 = 0x00CF9A000000FFFF;
const KERNEL_DATA_SEG: u64 = 0x00CF92000000FFFF;

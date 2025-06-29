#![feature(
    lang_items,
    adt_const_params,
    associated_type_defaults,
    core_intrinsics,
    unsized_const_params,
    strict_provenance_atomic_ptr
)]
#![allow(
    internal_features,
    incomplete_features,
    unused_variables,
    dead_code,
    unused_unsafe
)]
use core::intrinsics::AtomicOrdering;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::SeqCst;
include!("../common.rs");
extern crate core;
extern "C" {
    fn atomic_xor_u32(addr: &mut u32, xorand: u32) -> u32;
    fn atomic_nand_u32(addr: &mut u32, xorand: u32) -> u32;
    fn atomic_nand_u16(addr: &mut u16, xorand: u16) -> u16;
    fn atomic_nand_u8(addr: &mut u8, xorand: u8) -> u8;
    //fn atomic_cmpxchng_i32(addr: *mut i32, bytes: i32) -> i32;
}
use core::ptr::addr_of_mut;
//fn compare_exchange_byte(addr:&mut u8, byte:u8)->u8
fn main() {
    let mut u: u32 = black_box(20);
    let sub_old = unsafe {
        core::intrinsics::atomic_xsub::<_, { AtomicOrdering::SeqCst }>(addr_of_mut!(u), 10)
    };
    unsafe { printf(c"sub_old:%lx\n".as_ptr(), sub_old) };
    test_eq!(sub_old, 20);
    let mut u: u32 = black_box(20);
    let (val, is_eq) = unsafe {
        core::intrinsics::atomic_cxchgweak::<
            _,
            { AtomicOrdering::SeqCst },
            { AtomicOrdering::SeqCst },
        >(addr_of_mut!(u), 20_u32, 10)
    };
    test_eq!(val, 20_u32);
    test_eq!(u, 10_u32);
    //test_eq!(is_eq,true);
    let (val, is_eq) = unsafe {
        core::intrinsics::atomic_cxchgweak::<
            _,
            { AtomicOrdering::SeqCst },
            { AtomicOrdering::SeqCst },
        >(addr_of_mut!(u), 10_u32, 20)
    };
    test_eq!(val, 10_u32);
    let mut tmp = 0xFF_u32;
    unsafe { test_eq!(atomic_xor_u32(&mut tmp, 0x0A), 0xFF_u32) };
    test_eq!(tmp, 0xFF ^ 0x0A);
    let mut tmp = 0xFF_u32;
    unsafe { test_eq!(atomic_nand_u32(&mut tmp, 0x0A), 0xFF_u32) };
    test_eq!(tmp, !(0xFF & 0x0A));

    let mut tmp = 0xFF_u8;
    unsafe { test_eq!(atomic_nand_u8(&mut tmp, 0x0A_u8), 0xFF_u8) };
    unsafe { printf(c"%x\n".as_ptr(), tmp as u32) };
    test_eq!(tmp, !(0xFF_u8 & 0x0A_u8));

    let mut arr = [0x12, 0xFF, 0x45, 0x67];

    unsafe { test_eq!(atomic_nand_u8(&mut arr[1], 0x0A_u8), 0xFF_u8) };
    unsafe { printf(c"%x\n".as_ptr(), arr[1] as u32) };

    test_eq!(arr[0], 0x12);
    test_eq!(arr[1], !(0xFF_u8 & 0x0A_u8));
    test_eq!(arr[2], 0x45);
    test_eq!(arr[3], 0x67);

    /*let mut tmp = 0xFF_u16;
    unsafe { test_eq!(atomic_nand_u16(&mut tmp, 0x0A_u16), 0xFF_u16) };
    unsafe{printf(c"%x\n".as_ptr(),tmp as u32)};
    test_eq!(tmp, !(0xFF_u16 & 0x0A_u16));*/

    ptr_bitops_tagging();
    let atomic = core::sync::atomic::AtomicUsize::new(0);
    test_eq!(atomic.load(core::sync::atomic::Ordering::Relaxed), 0);
    let atomic_old = atomic.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    unsafe { printf(c"atomic_old:%lx\n".as_ptr(), atomic_old as u64) };
    test_eq!(atomic_old, 0);
    int_and();
}
fn ptr_bitops_tagging() {
    #[repr(align(16))]
    struct Tagme(#[allow(dead_code)] u128);

    let tagme = Tagme(1000);
    let ptr = &tagme as *const Tagme as *mut Tagme;
    let atom: AtomicPtr<Tagme> = AtomicPtr::new(ptr);

    const MASK_TAG: usize = 0b1111;
    const MASK_PTR: usize = !MASK_TAG;
    unsafe {
        printf(
            c"The 16 byte aligned tagme struct is located at an address of %p\n".as_ptr(),
            ptr.addr(),
        )
    };
    test_eq!(ptr.addr() & MASK_TAG, 0);

    test_eq!(atom.fetch_or(0b0111, SeqCst), ptr);
    test_eq!(atom.load(SeqCst), ptr.map_addr(|a| a | 0b111));

    test_eq!(
        atom.fetch_and(MASK_PTR | 0b0010, SeqCst),
        ptr.map_addr(|a| a | 0b111)
    );
    test_eq!(atom.load(SeqCst), ptr.map_addr(|a| a | 0b0010));

    // XOR not yet supported
    test_eq!(atom.fetch_xor(0b1011, SeqCst), ptr.map_addr(|a| a | 0b0010));
    test_eq!(atom.load(SeqCst), ptr.map_addr(|a| a | 0b1001));

    test_eq!(
        atom.fetch_and(MASK_PTR, SeqCst),
        ptr.map_addr(|a| a | 0b1001)
    );
    test_eq!(atom.load(SeqCst), ptr);
    ptr_add_data();
}
fn add_data() {
    let atom = AtomicPtr::<i64>::new(core::ptr::null_mut());
    test_eq!(atom.fetch_ptr_add(1, SeqCst).addr(), 0);
    test_eq!(atom.load(SeqCst).addr(), 8);

    test_eq!(atom.fetch_byte_add(1, SeqCst).addr(), 8);
    test_eq!(atom.load(SeqCst).addr(), 9);

    test_eq!(atom.fetch_ptr_sub(1, SeqCst).addr(), 9);
    test_eq!(atom.load(SeqCst).addr(), 1);

    test_eq!(atom.fetch_byte_sub(1, SeqCst).addr(), 1);
    test_eq!(atom.load(SeqCst).addr(), 0);
}
#[no_mangle]
fn ptr_add_data() {
    let num = 0i64;
    let n = &num as *const i64 as *mut _;
    let atom = AtomicPtr::<i64>::new(n);
    test_eq!(atom.fetch_ptr_add(1, SeqCst), n);
    test_eq!(atom.load(SeqCst), n.wrapping_add(1));

    test_eq!(atom.fetch_ptr_sub(1, SeqCst), n.wrapping_add(1));
    test_eq!(atom.load(SeqCst), n);
    let bytes_from_n = |b| n.wrapping_byte_add(b);

    test_eq!(atom.fetch_byte_add(1, SeqCst), n);
    test_eq!(atom.load(SeqCst), bytes_from_n(1));

    test_eq!(atom.fetch_byte_add(5, SeqCst), bytes_from_n(1));
    test_eq!(atom.load(SeqCst), bytes_from_n(6));

    test_eq!(atom.fetch_byte_sub(1, SeqCst), bytes_from_n(6));
    test_eq!(atom.load(SeqCst), bytes_from_n(5));

    test_eq!(atom.fetch_byte_sub(5, SeqCst), bytes_from_n(5));
    test_eq!(atom.load(SeqCst), n);
}
#[no_mangle]
fn int_and() {
    use std::sync::atomic::AtomicIsize;
    let x = AtomicIsize::new(0xf731);
    test_eq!(x.fetch_and(0x137f, SeqCst), 0xf731);
    test_eq!(x.load(SeqCst), 0xf731 & 0x137f);
}

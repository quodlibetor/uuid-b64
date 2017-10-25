#![feature(test)]

extern crate test;
extern crate uuid_b64;

use test::{Bencher, black_box};

use uuid_b64::UuidB64;


#[bench]
fn uuidb64_to_string(b: &mut Bencher) {
    let my_uuid = UuidB64::new();

    b.iter(|| black_box(my_uuid.to_string()));
}

#[bench]
fn uuidb64_to_inline_string(b: &mut Bencher) {
    let my_uuid = UuidB64::new();

    b.iter(|| black_box(my_uuid.to_istring()));
}


#[bench]
fn uuidb64_to_string_new_id_per_loop(b: &mut Bencher) {
    b.iter(|| {
        let my_uuid = UuidB64::new();
        black_box(my_uuid.to_string());
    });
}

#[bench]
fn uuidb64_to_inline_string_new_id_per_loop(b: &mut Bencher) {
    b.iter(|| {
        let my_uuid = UuidB64::new();
        black_box(my_uuid.to_istring());
    });
}


#[bench]
fn uuidb64_to_buf(b: &mut Bencher) {
    let my_uuid = UuidB64::new();
    let mut buf = String::new();

    b.iter(|| {
        my_uuid.to_buf(&mut buf);
        black_box(&buf);
    });
}

#[bench]
fn uuidb64_to_buf_new_id_per_loop(b: &mut Bencher) {
    let mut buf = String::new();
    b.iter(|| {
        let my_uuid = UuidB64::new();
        my_uuid.to_buf(&mut buf);
        black_box(&buf);
    });
}

#[bench]
fn uuidb64_to_buf_new_string_and_id_per_loop(b: &mut Bencher) {
    b.iter(|| {
        let my_uuid = UuidB64::new();
        let mut buf = String::new();
        my_uuid.to_buf(&mut buf);
        black_box(&buf);
    });
}

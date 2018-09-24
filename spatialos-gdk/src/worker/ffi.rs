#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use worker::schema::FieldId;
use std::slice;
use std::ffi::CString;

// TODO - These will leak. They need to be freed.
pub unsafe fn Schema_AddString(object: *mut Schema_Object, field_id: FieldId, value: &String) -> *const u8 {
    let cstring = CString::new(value.as_str()).unwrap();
    let raw_ptr = cstring.into_raw() as *const u8;
    Schema_AddBytes(
        object,
        field_id,
        raw_ptr,
        value.len() as u32,
    );
    raw_ptr
}

pub unsafe fn Schema_GetStringCount(object: *mut Schema_Object, field_id: FieldId) -> u32 {
    Schema_GetBytesCount(object, field_id)
}

pub unsafe fn Schema_IndexString(
    object: *mut Schema_Object,
    field_id: FieldId,
    index: u32,
) -> String {
    let count = Schema_IndexBytesLength(object, field_id, index) as usize;
    let pointer = Schema_IndexBytes(object, field_id, index) as *mut u8;

    // Need to copy memory to managed location
    let vec = slice::from_raw_parts(pointer, count).to_vec();
    String::from_utf8(vec).unwrap()
}

pub unsafe fn Schema_AddBoolean(object: *mut Schema_Object, field_id: FieldId, value: bool) {
    Schema_AddBool(object, field_id, if value {1}else{0});
}

pub unsafe fn Schema_GetBooleanCount(object: *mut Schema_Object, field_id: FieldId) -> u32 {
    Schema_GetBoolCount(object, field_id)
}

pub unsafe fn Schema_IndexBoolean(
    object: *mut Schema_Object,
    field_id: FieldId,
    index: u32,
) -> bool {
    Schema_IndexBool(object, field_id, index) != 0
}

// TODO - These will leak. They need to be freed.
pub unsafe fn Schema_AddBytesVec(object: *mut Schema_Object, field_id: FieldId, value: &Vec<u8>) -> *const u8 {
    let len = value.len() as u32;
    let value = value.clone().into_boxed_slice();
    let value_ptr = Box::into_raw(value) as *const u8;
    Schema_AddBytes(object, field_id, value_ptr, len);
    value_ptr
}

pub unsafe fn Schema_GetBytesVecCount(object: *mut Schema_Object, field_id: FieldId) -> u32 {
    Schema_GetBytesCount(object, field_id)
}

pub unsafe fn Schema_IndexBytesVec(
    object: *mut Schema_Object,
    field_id: FieldId,
    index: u32,
) -> Vec<u8> {
    let count = Schema_IndexBytesLength(object, field_id, index) as usize;
    let pointer = Schema_IndexBytes(object, field_id, index) as *mut u8;
    slice::from_raw_parts(pointer, count).to_vec()
}
/// Extension to compute primality of an Arrow array using Rust.
///
/// This module implements two functions that will be available from Python:
/// - `is_prime`: Returns an array with the primality of each individual number
/// - `are_all_primes`: Returns a scalar on whether all numbers are prime
///
/// DISCLAIMER: The code presented here is a simple example to illustrate
/// how to implement a pandas extension in Rust. It is not production code,
/// since for simplicity instead of properly controlling possible errors,
/// the code in most cases will panic. The code is also simplified to only
/// work with pandas Series of a single data type (uint64). Also, there are
/// simplifications on the prime number logic, like returning that the
/// primality of zero, one or null is false.
use pyo3::prelude::*;
use pyo3::ffi::Py_uintptr_t;
use arrow2::array::{UInt64Array, BooleanArray};
use arrow2::datatypes::{DataType, Field};
use arrow2::bitmap::MutableBitmap;
use arrow2::ffi;
use libc::uintptr_t;


/// Load the original Arrow array in pandas as a Rust Arrow2 array.
///
/// This is done by calling the `_export_to_c` function in the C++
/// implementation of Arrow (the implementation pandas uses) via FFI. The
/// function will create a struct with the relevant information
/// of the data (the memory address, the schema...). The data itself
/// is not copied, we access the original data. Only the struct with
/// the metadata is allocated.
pub fn pyarrow_to_arrow2(pyarrow_array: &PyAny) -> UInt64Array {
    let ffi_array = Box::new(ffi::ArrowArray::empty());
    let array_ptr = &*ffi_array as *const ffi::ArrowArray;

    let ffi_schema = Box::new(ffi::ArrowSchema::empty());
    let schema_ptr = &*ffi_schema as *const ffi::ArrowSchema;

    pyarrow_array.call_method1(
        "_export_to_c",
        (array_ptr as Py_uintptr_t, schema_ptr as Py_uintptr_t),
    ).unwrap();

    unsafe {
        let field = ffi::import_field_from_c(ffi_schema.as_ref()).unwrap();
        let array = ffi::import_array_from_c(*ffi_array, field.data_type).unwrap();

        if *array.data_type() != DataType::UInt64 {
            panic!("array type must be uint64");
        }
        array.as_any().downcast_ref::<UInt64Array>().unwrap().clone()
    }
}

pub fn arrow2_to_pyarrow(arrow2_array: BooleanArray, py: Python) -> PyResult<PyObject> {
    let pyarrow_mod = py.import("pyarrow")?;

    let arrow2_field = Field::new("is_prime", DataType::Boolean, false);

    let pyarrow_field = Box::new(ffi::export_field_to_c(&arrow2_field));
    let pyarrow_array = Box::new(ffi::export_array_to_c(arrow2_array.boxed()));    
    
    let schema_ptr: *const ffi::ArrowSchema = &*pyarrow_field;    
    let array_ptr: *const ffi::ArrowArray = &*pyarrow_array;

    let pyarrow_array = pyarrow_mod.getattr("Array")?
                                   .call_method1("_import_from_c",
                                                 (array_ptr as uintptr_t,
                                                  schema_ptr as uintptr_t))?;
    Ok(pyarrow_array.to_object(py))
}

/// Return whether an integer number is prime or not.
///
/// This is one of the common implementations to test primality.
/// Not the fastest implementation, but an efficient one.
///
/// Primality of 0 and 1 is undefined, but for simplicity we
/// return true for prime numbers, and false for everything else.
pub fn is_prime_scalar(n: u64) -> bool {
    if n == 0 || n == 1 { return false }
    if n == 2 || n == 3 { return true }
    if n % 2 == 0 || n % 3 == 0 { return false }

    let limit = (n as f64).powf(0.5).ceil() as u64;
    for i in (5..limit).step_by(6) {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
    }
    return true;
}

/// Check if every element of an array is prime.
///
/// The result is another array with true values in the positions
/// where a prime was found, and false in the rest.
///
/// This is the function that will be made accessible from Python,
/// so the input is a PyAny, that we expect to be a PyArrow
/// array of int64.
#[pyfunction]
fn is_prime(raw_pyarrow_array: &PyAny, py: Python) -> PyResult<PyObject> {
    let mut bitmap = MutableBitmap::with_capacity(raw_pyarrow_array.len().unwrap());

    for array_element in pyarrow_to_arrow2(raw_pyarrow_array).iter() {
        if let Some(&number) = array_element {
            bitmap.push(is_prime_scalar(number));
        } else {
            bitmap.push(false);
        }
    }
    let result = BooleanArray::new(DataType::Boolean, bitmap.into(), None);

    arrow2_to_pyarrow(result, py)
}

/// Check if all values in an array are prime.
///
/// Similar to `is_prime_array`, but returns a boolean scalar which
/// is true if all elements in the array are prime. This could be
/// implemented using the result of `is_prime_array`, but this
/// implementation is faster since it will stop early if any non-prime
/// is found.
///
/// It is implemented mostly to illustrate extension arrays that return
/// both a pandas Series or a Python scalar.
#[pyfunction]
fn are_all_primes(raw_pyarrow_array: &PyAny) -> PyResult<bool> {
    for array_element in pyarrow_to_arrow2(&raw_pyarrow_array).iter() {
        if let Some(number) = array_element {
            if !is_prime_scalar(*number) { return Ok(false) }
        }
    }
    Ok(true)
}

/// Python module that will be made available form Rust.
#[pymodule]
fn arrow_prime(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_prime, m)?)?;
    m.add_function(wrap_pyfunction!(are_all_primes, m)?)?;
    Ok(())
}

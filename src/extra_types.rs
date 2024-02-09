use pyo3::{pyclass, pymethods, types::PyModule, PyResult, Python};

macro_rules! build_python_type {
    ($st_name:ident, $rust_type:ty) => {
        #[pyclass]
        #[derive(Clone)]
        pub struct $st_name {
            inner_value: $rust_type,
        }

        impl $st_name {
            pub fn retrieve_value(&self) -> $rust_type {
                self.inner_value
            }
        }

        #[pymethods]
        impl $st_name {
            #[new]
            pub fn new_class(inner_value: $rust_type) -> Self {
                Self { inner_value }
            }

            pub fn __str__(&self) -> String {
                format!("{}, {}", stringify!($st_name), self.inner_value)
            }
        }
    };
}

build_python_type!(SmallInt, i16);
build_python_type!(Integer, i32);
build_python_type!(BigInt, i64);

pub fn extra_types_module(_py: Python<'_>, pymod: &PyModule) -> PyResult<()> {
    pymod.add_class::<SmallInt>()?;
    pymod.add_class::<Integer>()?;
    pymod.add_class::<BigInt>()?;
    Ok(())
}

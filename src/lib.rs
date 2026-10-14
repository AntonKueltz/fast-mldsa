use pyo3::{prelude::*, types::PyBytes};

use crate::dsa::{Dsa, MlDsa44, MlDsa65, MlDsa87};
use crate::signing_key::SigningKey;
use crate::verification_key::VerificationKey;

pub mod dsa;
mod integer_field;
mod matrix;
mod polynomial;
mod signing_key;
mod vector;
mod verification_key;

#[pyclass(from_py_object)]
#[derive(PartialEq, Clone, Copy)]
pub enum ParameterSet {
    #[pyo3(name = "ML_DSA_44")]
    MlDsa44 = 44,
    #[pyo3(name = "ML_DSA_65")]
    MlDsa65 = 65,
    #[pyo3(name = "ML_DSA_87")]
    MlDsa87 = 87,
}

enum VkInner {
    P44(VerificationKey<4, 4>),
    P65(VerificationKey<6, 5>),
    P87(VerificationKey<8, 7>),
}

enum SkInner {
    P44(SigningKey<4, 4>),
    P65(SigningKey<6, 5>),
    P87(SigningKey<8, 7>),
}

#[pyclass(name = "VerificationKey", frozen)]
struct PyVerificationKey {
    inner: VkInner,
}

#[pyclass(name = "SigningKey", frozen)]
struct PySigningKey {
    inner: SkInner,
}

#[pyclass(name = "ML_DSA", frozen)]
struct MlDsa {
    params: ParameterSet,
}

fn key_gen_impl<P: Dsa>() -> (P::VerKey, P::SigKey) {
    P::key_gen()
}

#[pymethods]
impl PyVerificationKey {
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        match &self.inner {
            VkInner::P44(k) => {
                let mut bytes = [0u8; 1312];
                k.to_bytes(&mut bytes);
                Ok(PyBytes::new(py, &bytes))
            }
            VkInner::P65(k) => {
                let mut bytes = [0u8; 1952];
                k.to_bytes(&mut bytes);
                Ok(PyBytes::new(py, &bytes))
            }
            VkInner::P87(k) => {
                let mut bytes = [0u8; 2592];
                k.to_bytes(&mut bytes);
                Ok(PyBytes::new(py, &bytes))
            }
        }
    }

    fn __bytes__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.to_bytes(py)
    }
}

#[pymethods]
impl PySigningKey {
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        match &self.inner {
            SkInner::P44(k) => {
                let mut bytes = [0u8; 2560];
                k.to_bytes(&mut bytes, 3);
                Ok(PyBytes::new(py, &bytes))
            }
            SkInner::P65(k) => {
                let mut bytes = [0u8; 4032];
                k.to_bytes(&mut bytes, 3);
                Ok(PyBytes::new(py, &bytes))
            }
            SkInner::P87(k) => {
                let mut bytes = [0u8; 4896];
                k.to_bytes(&mut bytes, 3);
                Ok(PyBytes::new(py, &bytes))
            }
        }
    }

    fn __bytes__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.to_bytes(py)
    }
}

#[pymethods]
impl MlDsa {
    #[new]
    #[pyo3(signature = (parameter_set = ParameterSet::MlDsa65))]
    fn new(parameter_set: ParameterSet) -> Self {
        Self {
            params: parameter_set,
        }
    }

    fn key_gen(&self) -> (PyVerificationKey, PySigningKey) {
        match self.params {
            ParameterSet::MlDsa44 => {
                let (pk, sk) = key_gen_impl::<MlDsa44>();
                (
                    PyVerificationKey {
                        inner: VkInner::P44(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P44(sk),
                    },
                )
            }
            ParameterSet::MlDsa65 => {
                let (pk, sk) = key_gen_impl::<MlDsa65>();
                (
                    PyVerificationKey {
                        inner: VkInner::P65(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P65(sk),
                    },
                )
            }
            ParameterSet::MlDsa87 => {
                let (pk, sk) = key_gen_impl::<MlDsa87>();
                (
                    PyVerificationKey {
                        inner: VkInner::P87(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P87(sk),
                    },
                )
            }
        }
    }
}

#[pymodule]
fn rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ParameterSet>()?;
    m.add_class::<PyVerificationKey>()?;
    m.add_class::<PySigningKey>()?;
    m.add_class::<MlDsa>()?;
    Ok(())
}

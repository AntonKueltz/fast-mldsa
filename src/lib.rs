use pyo3::exceptions::PyValueError;
use pyo3::{prelude::*, types::PyBytes};

use crate::dsa::{Dsa, MlDsa44, MlDsa65, MlDsa87};
use crate::signing_key::SigningKey;
use crate::verification_key::VerificationKey;

pub mod dsa;
mod hash;
mod integer;
mod matrix;
mod polynomial;
mod signer;
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

fn key_gen_impl<P: Dsa>(seed: Option<&[u8; 32]>) -> (P::VerKey, P::SigKey) {
    P::key_gen(seed)
}

fn sign_impl<'py, P: Dsa>(
    py: Python<'py>,
    sk: &P::SigKey,
    msg: &[u8],
    ctx: &[u8],
    deterministic: bool,
) -> PyResult<Bound<'py, PyBytes>> {
    match P::sign(sk, msg, ctx, deterministic) {
        Some(val) => Ok(PyBytes::new(py, val.as_ref())),
        None => Err(PyValueError::new_err("Error computing signature.")),
    }
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

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        match data.len() {
            1312 => Ok(Self {
                inner: VkInner::P44(
                    VerificationKey::<4, 4>::from_bytes(data)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            1952 => Ok(Self {
                inner: VkInner::P65(
                    VerificationKey::<6, 5>::from_bytes(data)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            2592 => Ok(Self {
                inner: VkInner::P87(
                    VerificationKey::<8, 7>::from_bytes(data)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            n => Err(PyValueError::new_err(format!(
                "Invalid verification key length {n}; expected 1312, 1952, or 2592"
            ))),
        }
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

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        match data.len() {
            2560 => Ok(Self {
                inner: SkInner::P44(
                    SigningKey::<4, 4>::from_bytes(data, 2)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            4032 => Ok(Self {
                inner: SkInner::P65(
                    SigningKey::<6, 5>::from_bytes(data, 4)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            4896 => Ok(Self {
                inner: SkInner::P87(
                    SigningKey::<8, 7>::from_bytes(data, 2)
                        .map_err(|e| PyValueError::new_err(e))?,
                ),
            }),
            n => Err(PyValueError::new_err(format!(
                "Invalid signing key length {n}; expected 2560, 4032, or 4896"
            ))),
        }
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

    #[pyo3(signature = (seed = None))]
    fn key_gen(&self, mut seed: Option<&[u8]>) -> PyResult<(PyVerificationKey, PySigningKey)> {
        if seed.as_mut().is_some_and(|x| x.len() != 32) {
            return Err(PyValueError::new_err("Seed must be exactly 32 bytes long"));
        }
        let seed_bytes: Option<&[u8; 32]> = seed.as_deref().and_then(|slice| slice.try_into().ok());

        match self.params {
            ParameterSet::MlDsa44 => {
                let (pk, sk) = key_gen_impl::<MlDsa44>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P44(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P44(sk),
                    },
                ))
            }
            ParameterSet::MlDsa65 => {
                let (pk, sk) = key_gen_impl::<MlDsa65>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P65(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P65(sk),
                    },
                ))
            }
            ParameterSet::MlDsa87 => {
                let (pk, sk) = key_gen_impl::<MlDsa87>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P87(pk),
                    },
                    PySigningKey {
                        inner: SkInner::P87(sk),
                    },
                ))
            }
        }
    }

    #[pyo3(signature = (sk, msg, ctx = None, deterministic = false))]
    fn sign<'py>(
        &self,
        py: Python<'py>,
        sk: &PySigningKey,
        msg: &[u8],
        ctx: Option<&[u8]>,
        deterministic: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let ctx = ctx.unwrap_or(&[]);

        match (self.params, &sk.inner) {
            (ParameterSet::MlDsa44, SkInner::P44(k)) => {
                sign_impl::<MlDsa44>(py, k, msg, ctx, deterministic)
            }
            (ParameterSet::MlDsa65, SkInner::P65(k)) => {
                sign_impl::<MlDsa65>(py, k, msg, ctx, deterministic)
            }
            (ParameterSet::MlDsa87, SkInner::P87(k)) => {
                sign_impl::<MlDsa87>(py, k, msg, ctx, deterministic)
            }
            _ => {
                return Err(PyValueError::new_err(
                    "Key does not match this ML_DSA parameter set",
                ));
            }
        }
    }

    #[pyo3(signature = (vk, sig, msg, ctx = None))]
    fn verify(
        &self,
        vk: &PyVerificationKey,
        sig: &[u8],
        msg: &[u8],
        ctx: Option<&[u8]>,
    ) -> PyResult<bool> {
        let ctx = ctx.unwrap_or(&[]);

        match (self.params, &vk.inner) {
            (ParameterSet::MlDsa44, VkInner::P44(k)) => Ok(MlDsa44::verify(k, msg, ctx, sig)),
            (ParameterSet::MlDsa65, VkInner::P65(k)) => Ok(MlDsa65::verify(k, msg, ctx, sig)),
            (ParameterSet::MlDsa87, VkInner::P87(k)) => Ok(MlDsa87::verify(k, msg, ctx, sig)),
            _ => {
                return Err(PyValueError::new_err(
                    "Key does not match this ML_DSA parameter set",
                ));
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

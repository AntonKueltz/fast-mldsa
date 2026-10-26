use pyo3::{
    buffer::PyBuffer,
    exceptions::PyValueError,
    prelude::*,
    types::{PyByteArray, PyBytes},
};
use zeroize::Zeroizing;

use ml_dsa::dsa::{Dsa, MlDsa44, MlDsa65, MlDsa87};
use ml_dsa::signing_key::SigningKey;
use ml_dsa::verification_key::VerificationKey;

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
    params: ParameterSet,
}

#[pyclass(name = "SigningKey", frozen)]
struct PySigningKey {
    inner: SkInner,
    params: ParameterSet,
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
                    VerificationKey::<4, 4>::from_bytes(data).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa44,
            }),
            1952 => Ok(Self {
                inner: VkInner::P65(
                    VerificationKey::<6, 5>::from_bytes(data).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa65,
            }),
            2592 => Ok(Self {
                inner: VkInner::P87(
                    VerificationKey::<8, 7>::from_bytes(data).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa87,
            }),
            _ => Err(PyValueError::new_err(
                "Invalid verification key length; expected 1312, 1952, or 2592",
            )),
        }
    }

    #[pyo3(signature = (sig, msg, ctx = None))]
    fn verify(&self, sig: &[u8], msg: &[u8], ctx: Option<&[u8]>) -> PyResult<bool> {
        let ctx = ctx.unwrap_or(&[]);

        match (self.params, &self.inner) {
            (ParameterSet::MlDsa44, VkInner::P44(k)) => Ok(MlDsa44::verify(k, msg, ctx, sig)),
            (ParameterSet::MlDsa65, VkInner::P65(k)) => Ok(MlDsa65::verify(k, msg, ctx, sig)),
            (ParameterSet::MlDsa87, VkInner::P87(k)) => Ok(MlDsa87::verify(k, msg, ctx, sig)),
            _ => Err(PyValueError::new_err(
                "Key does not match this ML_DSA parameter set",
            )),
        }
    }
}

#[pymethods]
impl PySigningKey {
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyByteArray>> {
        match &self.inner {
            SkInner::P44(k) => {
                let mut bytes = Zeroizing::new([0u8; 2560]);
                k.to_bytes(&mut *bytes, 2);
                PyByteArray::new_with(py, 2560, |b: &mut [u8]| {
                    b.copy_from_slice(&*bytes);
                    Ok(())
                })
            }
            SkInner::P65(k) => {
                let mut bytes = Zeroizing::new([0u8; 4032]);
                k.to_bytes(&mut *bytes, 4);
                PyByteArray::new_with(py, 4032, |b: &mut [u8]| {
                    b.copy_from_slice(&*bytes);
                    Ok(())
                })
            }
            SkInner::P87(k) => {
                let mut bytes = Zeroizing::new([0u8; 4896]);
                k.to_bytes(&mut *bytes, 2);
                PyByteArray::new_with(py, 4896, |b: &mut [u8]| {
                    b.copy_from_slice(&*bytes);
                    Ok(())
                })
            }
        }
    }

    fn __bytes__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        match &self.inner {
            SkInner::P44(k) => {
                let mut bytes = Zeroizing::new([0u8; 2560]);
                k.to_bytes(&mut *bytes, 2);
                Ok(PyBytes::new(py, &(*bytes)))
            }
            SkInner::P65(k) => {
                let mut bytes = Zeroizing::new([0u8; 4032]);
                k.to_bytes(&mut *bytes, 4);
                Ok(PyBytes::new(py, &(*bytes)))
            }
            SkInner::P87(k) => {
                let mut bytes = Zeroizing::new([0u8; 4896]);
                k.to_bytes(&mut *bytes, 2);
                Ok(PyBytes::new(py, &(*bytes)))
            }
        }
    }

    #[staticmethod]
    fn from_bytes(py: Python<'_>, data: PyBuffer<u8>) -> PyResult<Self> {
        let len = data.len_bytes();
        if !matches!(len, 2560 | 4032 | 4896) {
            return Err(PyValueError::new_err(
                "Invalid signing key length; expected 2560, 4032, or 4896",
            ));
        }

        let mut buf = Zeroizing::new([0u8; 4896]);
        data.copy_to_slice(py, &mut buf[..len])?;
        let bytes = &buf[..len];

        match len {
            2560 => Ok(Self {
                inner: SkInner::P44(
                    SigningKey::<4, 4>::from_bytes(bytes, 2).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa44,
            }),
            4032 => Ok(Self {
                inner: SkInner::P65(
                    SigningKey::<6, 5>::from_bytes(bytes, 4).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa65,
            }),
            4896 => Ok(Self {
                inner: SkInner::P87(
                    SigningKey::<8, 7>::from_bytes(bytes, 2).map_err(PyValueError::new_err)?,
                ),
                params: ParameterSet::MlDsa87,
            }),
            _ => unreachable!(),
        }
    }

    #[pyo3(signature = (msg, ctx = None, deterministic = false))]
    fn sign<'py>(
        &self,
        py: Python<'py>,
        msg: &[u8],
        ctx: Option<&[u8]>,
        deterministic: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let ctx = ctx.unwrap_or(&[]);

        match (self.params, &self.inner) {
            (ParameterSet::MlDsa44, SkInner::P44(k)) => {
                sign_impl::<MlDsa44>(py, k, msg, ctx, deterministic)
            }
            (ParameterSet::MlDsa65, SkInner::P65(k)) => {
                sign_impl::<MlDsa65>(py, k, msg, ctx, deterministic)
            }
            (ParameterSet::MlDsa87, SkInner::P87(k)) => {
                sign_impl::<MlDsa87>(py, k, msg, ctx, deterministic)
            }
            _ => Err(PyValueError::new_err(
                "Key does not match this ML_DSA parameter set",
            )),
        }
    }
}

#[pymethods]
impl MlDsa {
    fn __repr__(&self) -> &'static str {
        match self.params {
            ParameterSet::MlDsa44 => "ML-DSA-44",
            ParameterSet::MlDsa65 => "ML-DSA-65",
            ParameterSet::MlDsa87 => "ML-DSA-87",
        }
    }

    #[pyo3(signature = (seed = None))]
    fn key_gen(
        &self,
        py: Python<'_>,
        seed: Option<PyBuffer<u8>>,
    ) -> PyResult<(PyVerificationKey, PySigningKey)> {
        let mut seed_inner = Zeroizing::new([0u8; 32]);
        let seed_bytes: Option<&[u8; 32]> = match &seed {
            Some(b) => {
                if b.len_bytes() != 32 {
                    return Err(PyValueError::new_err("Seed must be exactly 32 bytes long"));
                }
                b.copy_to_slice(py, &mut *seed_inner)?;
                Some(&*seed_inner)
            }
            None => None,
        };

        match self.params {
            ParameterSet::MlDsa44 => {
                let (pk, sk) = key_gen_impl::<MlDsa44>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P44(pk),
                        params: ParameterSet::MlDsa44,
                    },
                    PySigningKey {
                        inner: SkInner::P44(sk),
                        params: ParameterSet::MlDsa44,
                    },
                ))
            }
            ParameterSet::MlDsa65 => {
                let (pk, sk) = key_gen_impl::<MlDsa65>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P65(pk),
                        params: ParameterSet::MlDsa65,
                    },
                    PySigningKey {
                        inner: SkInner::P65(sk),
                        params: ParameterSet::MlDsa65,
                    },
                ))
            }
            ParameterSet::MlDsa87 => {
                let (pk, sk) = key_gen_impl::<MlDsa87>(seed_bytes);
                Ok((
                    PyVerificationKey {
                        inner: VkInner::P87(pk),
                        params: ParameterSet::MlDsa87,
                    },
                    PySigningKey {
                        inner: SkInner::P87(sk),
                        params: ParameterSet::MlDsa87,
                    },
                ))
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
    m.add(
        "MlDsa",
        MlDsa {
            params: ParameterSet::MlDsa65,
        },
    )?;
    m.add(
        "MlDsa44",
        MlDsa {
            params: ParameterSet::MlDsa44,
        },
    )?;
    m.add(
        "MlDsa65",
        MlDsa {
            params: ParameterSet::MlDsa65,
        },
    )?;
    m.add(
        "MlDsa87",
        MlDsa {
            params: ParameterSet::MlDsa87,
        },
    )?;
    Ok(())
}

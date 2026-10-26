ML-DSA documentation
====================

This documentation is primarily focused on describing the package's API and usage. For details
on security, performance, implementation and development, see the
`README <https://github.com/AntonKueltz/fast-mldsa/blob/main/README.md>`_.

Examples
~~~~~~~~

.. code-block:: python

   from fast_mldsa import MlDsa, MlDsa87

   vk, sk = MlDsa.key_gen()  # verification and signing key for default parameter set
   msg, ctx = b"...", b"..."

   sig = sk.sign(msg, ctx)  # signature over the message and context
   valid = vk.verify(sig, msg, ctx)  # check if the signature is valid

   sig = sk.sign(msg)  # signature with no context

   sig = sk.sign(msg, deterministic=True)  # deterministic signature with no context

   vk, sk = MlDsa87.key_gen()  # keypair for ML-DSA-87 parameter set

   vk, sk = MlDsa(bytearray(32))  # keypair from seed of all 0s (don't do this, use a CSRNPG to generate seeds if you have to)


Reference
~~~~~~~~~

.. py:class:: ML_DSA

   A SUF-CMA module-lattice-based (ML) digital signature algorithm (DSA).

   This class cannot be instantiated. Instead, an instance of the class is provided for each
   parameter set defined by the ML-DSA standard - :code:`MlDsa44`, :code:`MlDsa65`, :code:`MlDsa87`.
   The instance :code:`MlDsa` is also provided, which will automatically use the default parameter
   set of ML-DSA-65.

   .. py:method:: ML_DSA.key_gen(seed: bytes | bytearray | memoryview | None = None) -> tuple[VerificationKey, SigningKey]:

      Generate a keypair (vk, sk) for verifying and creating ML-DSA signatures.

      The key generation algorithm accepts an optional seed as input. This can be useful in
      situation where e.g. network bandwidth is constrained and where providing the seed and
      re-deriving the keypair makes more sense than sending the byte encoded keypair. Note
      that the seed value must be exactly 32 bytes long. Since the seed is a sensitive value,
      it is recommended to use a mutable value for it that can be zero-ed out once it is no
      longer needed (like :type:`bytearray` or :type:`memoryview`). Recovery of the seed is
      equivalent to recovery of the signing key.

      :param seed: A 32 byte seed to derive the keypair from.
      :type seed: :type:`bytes | bytearray | memoryview | None`
      :return: The (verification key, signing key) pair.
      :rtype: :type:`tuple[VerificationKey, SigningKey]`

.. py:class:: SigningKey

   A signing key for ML-DSA.

   .. py:method:: SigningKey.sign(msg: bytes, ctx: bytes | None = None, deterministic: bool = False) -> bytes:

      Generate a signature for the given arguments using the given signing key.

      :param msg: A message to sign.
      :type msg: :type:`bytes`
      :param ctx: Optional additional context for the message. If ommitted, equivalent to an empty byte sequence.
      :type ctx: :type:`bytes | None`
      :param deterministic: If true, signature is not randomized meaning the same message and context inputs produce the same signature.
      :type deterministic: :type:`bool`
      :return: The byte encoded signature.
      :rtype: :type:`bytes`

   .. py:staticmethod:: SigningKey.from_bytes(data: bytes | bytearray | memoryview) -> SigningKey:

      Deserialize a signing key from its byte encoding.

      It is recommended to use :type:`bytearray` or :type:`memoryview` to represent the encoded key
      as it is secret material that the application may want to zero out after using.

      :param data: The byte encoding of the signing key.
      :type data: :type:`bytes | bytearray | memoryview`
      :return: The deserialized signing key.
      :rtype: :type:`SigningKey`

   .. py:method:: SigningKey.to_bytes() -> bytearray:

      Serialize the signing key to its byte encoding.

      Note that this returns a mutable :type:`bytearray` rather than :type:`bytes` in case the
      application needs to zero out the encoded key after using it.

      :return: The serialized signing key.
      :rtype: :type:`bytearray`

   .. py:method:: SigningKey.__bytes__() -> bytes:

      Serialize the signing key to its byte encoding.

      Note that this returns immutable :type:`bytes` which cannot be zero-ed out and may remain
      in memory even after garbage collection / :code:`dealloc` is called.

      :return: The serialized signing key.
      :rtype: :type:`bytes`

.. py:class:: VerificationKey

   A verification key for ML-DSA.

   .. py:method:: VerificationKey.verify(sig: bytes, msg: bytes, ctx: bytes | None = None) -> bool:

      Verify the signature for the given arguments using the given verification key.

      :param sig: A byte encoded signature.
      :type sig: :type:`bytes`
      :param msg: A message corresponding to the signature.
      :type msg: :type:`bytes`
      :param ctx: Optional additional context for the message. If ommitted, equivalent to an empty byte sequence.
      :type ctx: :type:`bytes | None`
      :return: Whether or not the signature is valid.
      :rtype: :type:`bool`

   .. py:staticmethod:: VerificationKey.from_bytes(data: bytes) -> VerificationKey:

      Deserialize a verification key from its byte encoding.

      :param data: The byte encoding of the verification key.
      :type data: :type:`bytes`
      :return: The deserialized verification key.
      :rtype: :type:`VerificationKey`

   .. py:method:: VerificationKey.to_bytes() -> bytes:

      Serialize the verification key to its byte encoding.

      :return: The serialized verification key.
      :rtype: :type:`bytes`

   .. py:method:: VerificationKey.__bytes__() -> bytes:

      Serialize the verification key to its byte encoding.

      :return: The serialized verification key.
      :rtype: :type:`bytes`

use shake::{ExtendableOutput, Shake256, Update, XofReader};

pub(crate) fn h(out: &mut [u8], chunks: &[&[u8]]) {
    let mut shaker = Shake256::default();

    for chunk in chunks {
        shaker.update(*chunk);
    }

    shaker.finalize_xof().read(out)
}

#![doc(html_favicon_url = "https://imgur.com/ZEp8ZTS.png")]
#![doc(html_logo_url = "https://i.imgur.com/oWYCiKJ.png")]

pub mod spaces;
pub use spaces::Space;

use pyo3::prelude::*;

#[pymodule]
fn free_range_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Space>()?;
    Ok(())
}

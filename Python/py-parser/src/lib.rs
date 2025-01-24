use config_deserializer::ParserConfigDeserializer;
use parser::{config::ParserConfig, passage_state::PassageState};
use pyo3::prelude::*;

#[pyclass]
pub struct Passage(pub PassageState);

#[pymethods]
impl Passage {
    #[new]
    pub fn new(input: &str, config: &str) -> Self {
        let config: ParserConfig = ParserConfigDeserializer::from_toml_str(config)
            .unwrap()
            .into();
        Self(PassageState::new(input, config))
    }

    pub fn locate_fragment(&self, fragment: &str) -> (isize, isize) {
        self.0
            .locate_fragment(fragment)
            .map(|(start, end)| (start as isize, end as isize))
            .unwrap_or((-1, -1))
    }

    pub fn get_raw_passage(&self) -> String {
        self.0.get_raw_passage()
    }

    pub fn is_valid(&self) -> bool {
        self.0.valid
    }
}

#[pymodule]
fn py_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Passage>()?;
    Ok(())
}

//

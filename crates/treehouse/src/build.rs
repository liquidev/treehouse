use std::path::{Path, PathBuf};

pub trait Task {
    fn name(&self) -> &'static str;
    fn inputs(&self) -> &[TargetId];
    fn run(&self, output_path: &Path) -> anyhow::Result<()>;
}

pub struct Build {
    targets: Vec<Target>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetId(usize);

struct Target {
    output: PathBuf,
    task: Box<dyn Task>,
}

impl Build {
    pub fn new() -> Self {
        Self { targets: vec![] }
    }

    pub fn add(&mut self, output_path: impl Into<PathBuf>, task: impl Task + 'static) -> TargetId {
        self.add_inner(output_path.into(), Box::new(task))
    }

    fn add_inner(&mut self, output_path: PathBuf, task: Box<dyn Task>) -> TargetId {}

    pub fn run(&self, target: TargetId) -> anyhow::Result<()> {
        Ok(())
    }
}

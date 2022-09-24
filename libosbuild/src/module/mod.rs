#[derive(Debug)]
pub enum AssemblerError {}

pub trait Assembler {}

#[derive(Debug)]
pub enum SourceError {}

pub trait Source {
    fn cached(&self) -> Result<bool, SourceError>;

    fn fetch_all(&self) -> Result<(), SourceError>;
    fn fetch_one(&self) -> Result<(), SourceError>;
}

#[derive(Debug)]
pub enum StageError {}

pub trait Stage {}

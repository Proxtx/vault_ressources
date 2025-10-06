use std::path::Path;

pub trait RessourceType {
    fn id() -> &'static str;
}

pub trait ReadableRessource: RessourceType
where
    Self::Error: 'static,
{
    type Error: std::error::Error;
    fn read(path: &Path) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;
}

pub trait WritableRessource: RessourceType
where
    Self::Error: 'static,
{
    type Error: std::error::Error;
    fn data_extension() -> &'static str;
    fn write(&self, path: &Path) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

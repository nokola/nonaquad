use thiserror::Error;

#[derive(Error, Debug)]
pub enum NonaError {
    #[error("ERR_TEXTURE: {0}")]
    Texture(String),

    #[error("ERR_SHADER: {0}")]
    Shader(String),

    #[error("ERR_FONT: {0}")]
    Font(String),
}

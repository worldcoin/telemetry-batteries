pub mod json_eyre;

pub enum EyreMode {
    ColorEyre,
    JsonEyre,
}

pub struct EyreConfig {
    mode: EyreMode,
    with_default_backtrace: bool,
    with_default_spantrace: bool,
}

impl Default for EyreConfig {
    fn default() -> Self {
        Self {
            mode: EyreMode::ColorEyre,
            with_default_backtrace: true,
            with_default_spantrace: true,
        }
    }
}

pub struct EyreBattery;

impl EyreBattery {
    pub fn init(config: EyreConfig) -> eyre::Result<()> {
        match config.mode {
            EyreMode::ColorEyre => {
                color_eyre::install()?;
                Ok(())
            }
            EyreMode::JsonEyre => {
                json_eyre::install(
                    config.with_default_backtrace,
                    config.with_default_spantrace,
                )?;
                Ok(())
            }
        }
    }
}

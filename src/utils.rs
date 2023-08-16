use crate::interlude::*;

pub fn setup_tracing() -> eyre::Result<()> {
    color_eyre::install()?;
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        // .pretty()
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .try_init()
        .map_err(|err| eyre::anyhow!(err))?;

    // tracing_log::LogTracer::init()?;

    Ok(())
}

pub fn get_env_var<K>(key: K) -> eyre::Result<String>
where
    K: AsRef<std::ffi::OsStr>,
{
    match std::env::var(key.as_ref()) {
        Ok(val) => Ok(val),
        Err(err) => Err(eyre::eyre!(
            "error geting env var {:?}: {err}",
            key.as_ref()
        )),
    }
}

pub trait ResultExt<T, E> {
    fn map_eyre(self) -> eyre::Result<T>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Into<eyre::Report>,
{
    fn map_eyre(self) -> eyre::Result<T> {
        self.map_err(|err| eyre::eyre!(err))
    }
}

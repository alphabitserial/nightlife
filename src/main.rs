use futures_util::stream::StreamExt;
use zbus::zvariant::OwnedValue;
use zbus::{proxy, Connection, Result};

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait PortalSettings {
    /// SettingChanged signal
    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: OwnedValue) -> Result<()>;

    /// Read a setting value
    fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue>;
}

#[derive(Debug, Clone, Copy)]
enum ThemePreference {
    NoPreference,
    Dark,
    Light,
    Unknown,
}

impl From<u32> for ThemePreference {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::NoPreference,
            1 => Self::Dark,
            2 => Self::Light,
            _ => Self::Unknown,
        }
    }
}

fn parse_theme_preference(value: &OwnedValue) -> ThemePreference {
    // The value is a variant containing a u32
    value
        .downcast_ref::<u32>()
        .map(|val| ThemePreference::from(val))
        .map_err(|_| ThemePreference::Unknown)
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;

    let proxy = PortalSettingsProxy::new(&connection).await?;

    // Try to read current theme preference
    match proxy
        .read("org.freedesktop.appearance", "color-scheme")
        .await
    {
        Ok(current_value) => {
            let theme = parse_theme_preference(&current_value);
            println!("Current theme preference: {:?}", theme);
        }
        Err(e) => println!("Could not read current theme preference: {}", e),
    }

    // Listen for theme changes
    let mut stream = proxy.receive_setting_changed().await?;
    println!("Listening for theme changes...");

    while let Some(signal) = stream.next().await {
        let args = signal.args()?;

        // Check if this is an appearance setting change
        if args.namespace == "org.freedesktop.appearance" && args.key == "color-scheme" {
            let theme = parse_theme_preference(&args.value);
            println!("Theme changed to: {:?}", theme);
        }
    }

    Ok(())
}

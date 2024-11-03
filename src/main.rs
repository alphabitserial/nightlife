use std::fs;
use std::path::PathBuf;
use std::process::Command;

const DARK_THEME: &str = "modus_vivendi";
const LIGHT_THEME: &str = "modus_operandi";

fn main() {
    // home_dir is deprecated because of possible bad behavior on Windows
    // we don't care because this tool already assumes a Linux environment
    #[allow(deprecated)]
    let home_dir = std::env::home_dir().expect("find home directory");
    let is_dark = is_dark(&mut home_dir.clone());
    set_hx_theme(&mut home_dir.clone(), is_dark);
    set_kitty_theme(&mut home_dir.clone(), is_dark);
}

fn is_dark(path: &mut PathBuf) -> bool {
    path.push(".config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark");
    let cosmic_dark = fs::read_to_string(path).expect("read cosmic dark mode setting");
    cosmic_dark
        .parse()
        .expect("cosmic setting is_dark is always true or false")
}

fn set_hx_theme(path: &mut PathBuf, is_dark: bool) {
    path.push(".config/helix/config.toml");
    let conf = fs::read_to_string(&path).expect("read hx config");

    let mut new_conf = String::new();
    for line in conf.lines() {
        if line.starts_with("theme = ") {
            new_conf.push_str("theme = \"");
            new_conf.push_str({
                if is_dark {
                    DARK_THEME
                } else {
                    LIGHT_THEME
                }
            });
            new_conf.push_str("\"\n");
        } else {
            new_conf.push_str(line);
            new_conf.push('\n');
        }
    }

    fs::write(&path, new_conf.as_bytes()).expect("write hx config");
}

fn set_kitty_theme(path: &mut PathBuf, is_dark: bool) {
    path.push(".config/kitty/");
    let symlink_name = "current-theme.conf";
    let theme_file = path.join("themes").join({
        if is_dark {
            DARK_THEME
        } else {
            LIGHT_THEME
        }
    });

    // we must remove the existing symlink first to avoid an fs error
    fs::remove_file(path.join(symlink_name)).expect("remove symlink");
    std::os::unix::fs::symlink(theme_file, path.join(symlink_name)).expect("symlink theme");

    // https://sw.kovidgoyal.net/kitty/conf/
    // send SIGUSR1 to all kitty processes to force config reload
    Command::new("killall")
        .args(["-s", "SIGUSR1", "kitty"])
        .spawn()
        .expect("reload kitty config");
}

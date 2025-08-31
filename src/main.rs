use dialoguer::Select;
use std::fs::{remove_file, File};
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::{env, io, panic};

/// URL to download yt-dlp from
const YT_DLP_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
/// List of browsers that yt-dlp supports for `--cookies-from-browser`
const YT_DLP_BROWSERS: [&str; 9] = [
	"brave", "chrome", "chromium", "edge", "firefox", "opera", "safari", "vivaldi", "whale",
];

fn main() -> anyhow::Result<()> {
	// user-friendly panic handling
	panic::set_hook(Box::new(|info| {
		if let Some(str) = info.payload().downcast_ref::<&str>() {
			println!("Fatal error: {str}");
		} else if let Some(string) = info.payload().downcast_ref::<String>() {
			println!("Fatal error: {string}");
		} else {
			println!("Fatal error: (no message provided)");
		}

		sleep(Duration::MAX);
	}));

	// VRChat stores yt-dlp.exe in %LOCALAPPDATA%Low\VRChat\VRChat\Tools\
	let mut tools_dir = env::var("LOCALAPPDATA")?;
	tools_dir.push_str(r"Low\VRChat\VRChat\Tools\");

	// yt-dlp.conf is stored in the same folder as yt-dlp.exe
	let conf_path = format!("{tools_dir}yt-dlp.conf");
	let mut conf_file = File::create(conf_path).expect("failed to create yt-dlp.conf");
	println!("Created yt-dlp.conf");

	// select browser, write to yt-dlp.conf
	let browser = Select::new()
		.with_prompt("Select the browser that you use")
		.items(&YT_DLP_BROWSERS)
		.interact()?;
	let browser = YT_DLP_BROWSERS[browser];
	conf_file
		.write_all(format!("--cookies-from-browser {browser} -t sleep").as_bytes())
		.expect("failed to write to yt-dlp.conf");
	println!("Wrote to yt-dlp.conf");

	// delete old yt-dlp.exe
	let exe_path = format!("{tools_dir}yt-dlp.exe");
	remove_file(&exe_path).expect("failed to delete yt-dlp.exe");
	println!("Deleted yt-dlp.exe");

	// create new yt-dlp.exe
	let mut exe_file = File::create_new(&exe_path).expect("failed to create yt-dlp.exe");
	println!("Created yt-dlp.exe");

	// download yt-dlp.exe
	let mut response = reqwest::blocking::get(YT_DLP_URL).expect("failed to download yt-dlp.exe");
	println!("Downloaded yt-dlp.exe");

	// write to yt-dlp.exe
	io::copy(&mut response, &mut exe_file).expect("failed to write to yt-dlp.exe");
	println!("Wrote to yt-dlp.exe");

	// set permissions for yt-dlp.exe
	let mut exe_file_permissions = exe_file.metadata()?.permissions();
	exe_file_permissions.set_readonly(true);
	exe_file
		.set_permissions(exe_file_permissions)
		.expect("failed to set permissions for yt-dlp.exe");
	println!("Set permissions for yt-dlp.exe");

	// set DACLs for yt-dlp.exe
	Command::new("icacls")
		.args([&exe_path, "/setintegritylevel", "medium", "/inheritance:d"])
		.output()
		.expect("failed to set DACLs for yt-dlp.exe");
	println!("Set DACLs for yt-dlp.exe");

	// done
	println!("Done! You can close this window.");
	sleep(Duration::MAX);

	Ok(())
}

use dialoguer::Select;
use std::fs::{remove_file, File};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use std::{env, io, panic};
use std::process::Command;
use tempfile::tempfile;
use zip::ZipArchive;

/// yt-dlp download URL
const YT_DLP_URL: &str = "https://github.com/yt-dlp/yt-dlp-nightly-builds/releases/latest/download/yt-dlp.exe";
/// Base `yt-dlp.conf`
const YT_DLP_CONF: &str = r"-t sleep
--no-playlist
--impersonate safari
--extractor-args youtube:player_client=web
--cookies-from-browser ";
/// List of browsers that yt-dlp supports for `--cookies-from-browser`
const YT_DLP_SUPPORTED_BROWSERS: [&str; 9] = [
	"brave", "chrome", "chromium", "edge", "firefox", "opera", "safari", "vivaldi", "whale",
];
/// Deno download URL
const DENO_URL: &str = "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip";

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
	let tools_dir = format!(r"{}Low\VRChat\VRChat\Tools\", env::var("LOCALAPPDATA")?);

	// select browser, write to yt-dlp.conf
	let browser = Select::new()
		.with_prompt("Select the browser that you use")
		.items(&YT_DLP_SUPPORTED_BROWSERS)
		.interact()?;
	let browser = YT_DLP_SUPPORTED_BROWSERS[browser];

	// create yt-dlp.conf
	let conf_path = format!("{tools_dir}yt-dlp.conf");
	let mut conf_file = File::create(conf_path).expect("failed to create yt-dlp.conf");
	println!("Created yt-dlp.conf");

	// write to yt-dlp.conf
	let mut conf = String::from(YT_DLP_CONF);
	conf.push_str(browser);
	conf.push('\n');
	conf_file
		.write_all(conf.as_bytes())
		.expect("failed to write to yt-dlp.conf");
	println!("Wrote to yt-dlp.conf");

	// delete yt-dlp.exe
	let exe_path = format!("{tools_dir}yt-dlp.exe");
	remove_file(&exe_path).expect("failed to delete yt-dlp.exe");
	println!("Deleted yt-dlp.exe");

	// create new yt-dlp.exe
	let mut exe_file = File::create_new(&exe_path).expect("failed to create yt-dlp.exe");

	// download and write to yt-dlp.exe
	io::copy(
		&mut reqwest::blocking::get(YT_DLP_URL).expect("failed to download yt-dlp.exe"),
		&mut exe_file,
	)
	.expect("failed to write to yt-dlp.exe");
	println!("Downloaded yt-dlp.exe");

	// set permissions on yt-dlp.exe
	let mut exe_file_permissions = exe_file.metadata()?.permissions();
	exe_file_permissions.set_readonly(true);
	exe_file
		.set_permissions(exe_file_permissions)
		.expect("failed to set permissions on yt-dlp.exe");

	// set DACLs on yt-dlp.exe
	Command::new("icacls")
		.args([&exe_path, "/setintegritylevel", "medium", "/inheritance:d"])
		.output()
		.expect("failed to set DACLs on yt-dlp.exe");

	// create deno.exe
	let deno_path = format!("{tools_dir}deno.exe");
	let mut deno_file = File::create(&deno_path).expect("failed to create deno.exe");

	// create deno.exe.zip
	let mut deno_zip_file = tempfile().expect("failed to create deno.exe.zip");

	// download and write to deno.exe.zip
	io::copy(
		&mut reqwest::blocking::get(DENO_URL).expect("failed to download deno.exe.zip"),
		&mut deno_zip_file,
	)
	.expect("failed to write to deno.exe.zip");

	// unzip and write to deno.exe
	io::copy(&mut ZipArchive::new(deno_zip_file)?.by_index(0)?, &mut deno_file).expect("failed to write to deno.exe");
	println!("Downloaded deno.exe");

	// done
	println!("Done! You can close this window.");
	sleep(Duration::MAX);

	Ok(())
}

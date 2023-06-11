// copy rinna.exe to target dir
/*
fn copy_rinna_exe() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        println!("cargo:warning=OUT_DIR not found");
        return;
    };
    let Ok(_) = std::fs::copy("./rinna.exe", format!("{}/../../../rinna.exe", out_dir)) else {
        println!("cargo:warning=Failed to copy rinna.exe");
        return;
    };
}
*/

fn main() {
    // copy_rinna_exe();

    tauri_build::build()
}

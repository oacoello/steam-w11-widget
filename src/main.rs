#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adaptive_card;
mod i18n;
mod provider;
mod steam;
mod widget_bindings;

use i18n::{detect_system_lang, get_strings};
use provider::{log_msg, WidgetProviderFactory, WIDGET_PROVIDER_CLSID};
use steam::{format_playtime, format_size, scan_installed_games};
use windows::Win32::System::Com::{
    CoInitializeEx, CoRegisterClassObject, CoResumeClassObjects, CoRevokeClassObject,
    CoUninitialize, IClassFactory, CLSCTX_LOCAL_SERVER, COINIT_MULTITHREADED, REGCLS_MULTIPLEUSE,
    REGCLS_SUSPENDED,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};
use windows_core::PCWSTR;

#[link(name = "Kernel32")]
extern "system" {
    fn LoadLibraryExW(
        lpLibFileName: *const u16,
        hFile: *mut std::ffi::c_void,
        dwFlags: u32,
    ) -> *mut std::ffi::c_void;
}

fn ensure_windows_app_runtime_loaded() {
    unsafe {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                let dir_str = format!("{}\\", dir.to_string_lossy());
                std::env::set_var("MICROSOFT_WINDOWSAPPRUNTIME_BASE_DIRECTORY", &dir_str);

                let dll_path = dir.join("Microsoft.WindowsAppRuntime.dll");
                let mut wide: Vec<u16> = dll_path
                    .to_string_lossy()
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                let hmod = LoadLibraryExW(wide.as_mut_ptr(), std::ptr::null_mut(), 0);
                log_msg(&format!(
                    "LoadLibraryExW(Microsoft.WindowsAppRuntime.dll) -> {:p}",
                    hmod
                ));
            }
        }
    }
}

fn run_com_server() -> windows_core::Result<()> {
    log_msg("Starting Steam Windows 11 Widget COM Server...");
    ensure_windows_app_runtime_loaded();

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

        let factory: IClassFactory = WidgetProviderFactory::new().into();
        let cookie = CoRegisterClassObject(
            &WIDGET_PROVIDER_CLSID,
            &factory,
            CLSCTX_LOCAL_SERVER,
            REGCLS_MULTIPLEUSE | REGCLS_SUSPENDED,
        )?;
        CoResumeClassObjects()?;

        log_msg(&format!(
            "Registered COM ClassFactory for CLSID {:?} (cookie={})",
            WIDGET_PROVIDER_CLSID, cookie
        ));

        let stop_event = CreateEventW(None, true, false, PCWSTR::null())?;
        let _ = WaitForSingleObject(stop_event, INFINITE);

        let _ = CoRevokeClassObject(cookie);
        CoUninitialize();
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    log_msg(&format!("Process launched with args: {:?}", args));

    let is_com_activation = args.iter().any(|a| {
        let lower = a.to_lowercase();
        lower.contains("registerprocessascomserver") || lower.contains("embedding")
    });

    if args.iter().any(|a| a == "--scan" || a == "--test") {
        let strings = get_strings(detect_system_lang());
        let games = scan_installed_games();
        println!("Juegos instalados detectados en Steam ({}):", games.len());
        for g in &games {
            log_msg(&format!(
                "Scanned Game: [{}] {} | {} | {}",
                g.appid,
                g.name,
                format_playtime(g.playtime_minutes, &strings),
                format_size(g.size_on_disk, &strings)
            ));
        }
        let (tmpl, data) = adaptive_card::build_adaptive_card(
            &games,
            0,
            widget_bindings::WidgetSize::Medium,
            None,
        );
        println!(
            "\nAdaptive Card generada correctamente (Template: {} bytes, Data: {} bytes)",
            tmpl.len(),
            data.len()
        );
        return;
    }

    if is_com_activation || args.len() <= 1 {
        if !is_com_activation {
            let strings = get_strings(detect_system_lang());
            println!("=== Steam Library Widget Provider para Windows 11 ===");
            let games = scan_installed_games();
            println!("Juegos instalados detectados ({}):", games.len());
            for g in &games {
                println!(
                    "  • {} (AppID: {}, Tamaño: {})",
                    g.name,
                    g.appid,
                    format_size(g.size_on_disk, &strings)
                );
            }
            println!("\nIniciando servidor COM del proveedor de Widgets (Presiona Ctrl+C para salir)...");
        }
        if let Err(e) = run_com_server() {
            log_msg(&format!("COM Server error: {:?}", e));
            eprintln!("Error en el servidor COM: {:?}", e);
        }
    }
}

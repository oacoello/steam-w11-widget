#![allow(non_snake_case)]

use crate::adaptive_card::build_adaptive_card;
use crate::steam::{launch_steam_game, open_steam_library, scan_installed_games, SteamGame};
use crate::widget_bindings::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{CLASS_E_CLASSNOTAVAILABLE, E_NOINTERFACE, E_POINTER, S_OK};
use windows::Win32::System::Com::{CoTaskMemAlloc, IClassFactory, IClassFactory_Impl};
use windows_core::*;

pub const WIDGET_PROVIDER_CLSID: GUID = GUID::from_u128(0xB8A9C7E1_4D32_4E9A_9B1C_7F5E3A2D1C08);

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WidgetCustomState {
    pub page: usize,
}

#[derive(Clone, Debug)]
struct WidgetInstance {
    size: WidgetSize,
    page: usize,
}

struct ProviderState {
    widgets: HashMap<String, WidgetInstance>,
    games: Vec<SteamGame>,
}

pub fn log_msg(msg: &str) {
    if let Ok(temp) = std::env::var("TEMP") {
        let log_path = std::path::PathBuf::from(temp).join("steam_w11_widget.log");
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

#[repr(C)]
pub struct SteamWidgetProvider {
    vtable: *const IWidgetProvider_Vtbl,
    ref_count: AtomicU32,
    state: Arc<Mutex<ProviderState>>,
}

unsafe impl Send for SteamWidgetProvider {}
unsafe impl Sync for SteamWidgetProvider {}

impl SteamWidgetProvider {
    const VTABLE: IWidgetProvider_Vtbl = IWidgetProvider_Vtbl {
        base__: IInspectable_Vtbl {
            base: IUnknown_Vtbl {
                QueryInterface: Self::qi,
                AddRef: Self::add_ref,
                Release: Self::release,
            },
            GetIids: Self::get_iids,
            GetRuntimeClassName: Self::get_runtime_class_name,
            GetTrustLevel: Self::get_trust_level,
        },
        CreateWidget: Self::create_widget,
        DeleteWidget: Self::delete_widget,
        OnActionInvoked: Self::on_action_invoked,
        OnWidgetContextChanged: Self::on_widget_context_changed,
        Activate: Self::activate,
        Deactivate: Self::deactivate,
    };

    pub fn new_boxed() -> *mut Self {
        log_msg("SteamWidgetProvider::new_boxed() initializing...");
        let games = scan_installed_games();
        log_msg(&format!("Scanned {} installed Steam games", games.len()));

        let provider = Box::new(Self {
            vtable: &Self::VTABLE,
            ref_count: AtomicU32::new(1),
            state: Arc::new(Mutex::new(ProviderState {
                widgets: HashMap::new(),
                games,
            })),
        });
        let raw = Box::into_raw(provider);
        unsafe {
            (*raw).recover_existing_widgets();
        }
        raw
    }

    unsafe extern "system" fn qi(
        this: *mut c_void,
        iid: *const GUID,
        interface: *mut *mut c_void,
    ) -> HRESULT {
        if iid.is_null() || interface.is_null() {
            return E_POINTER;
        }
        let iid_val = *iid;
        log_msg(&format!("SteamWidgetProvider::QI({:?})", iid_val));

        if iid_val == IUnknown::IID
            || iid_val == IInspectable::IID
            || iid_val == IWidgetProvider::IID
        {
            *interface = this;
            Self::add_ref(this);
            return S_OK;
        }

        *interface = std::ptr::null_mut();
        E_NOINTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
        let me = &*(this as *const Self);
        me.ref_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    unsafe extern "system" fn release(this: *mut c_void) -> u32 {
        let me = &*(this as *const Self);
        let prev = me.ref_count.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(this as *mut Self));
            return 0;
        }
        prev - 1
    }

    unsafe extern "system" fn get_iids(
        _this: *mut c_void,
        count: *mut u32,
        values: *mut *mut GUID,
    ) -> HRESULT {
        if count.is_null() || values.is_null() {
            return E_POINTER;
        }
        log_msg("SteamWidgetProvider::GetIids called!");
        let mem = CoTaskMemAlloc(std::mem::size_of::<GUID>()) as *mut GUID;
        if mem.is_null() {
            *count = 0;
            *values = std::ptr::null_mut();
            return HRESULT(0x8007000Eu32 as i32); // E_OUTOFMEMORY
        }
        *mem = IWidgetProvider::IID;
        *count = 1;
        *values = mem;
        S_OK
    }

    unsafe extern "system" fn get_runtime_class_name(
        _this: *mut c_void,
        value: *mut *mut c_void,
    ) -> HRESULT {
        if value.is_null() {
            return E_POINTER;
        }
        log_msg("SteamWidgetProvider::GetRuntimeClassName called!");
        let name = HSTRING::from("Microsoft.Windows.Widgets.Providers.IWidgetProvider");
        *value = std::mem::transmute::<HSTRING, *mut c_void>(name);
        S_OK
    }

    unsafe extern "system" fn get_trust_level(_this: *mut c_void, value: *mut i32) -> HRESULT {
        if value.is_null() {
            return E_POINTER;
        }
        log_msg("SteamWidgetProvider::GetTrustLevel called!");
        *value = 0; // BaseTrust
        S_OK
    }

    unsafe extern "system" fn create_widget(
        this: *mut c_void,
        widget_context: *mut c_void,
    ) -> HRESULT {
        let me = &*(this as *const Self);
        if widget_context.is_null() {
            return S_OK;
        }
        if let Some(ctx) = IWidgetContext::from_raw_borrowed(&widget_context) {
            let id = ctx.id().unwrap_or_default();
            let size = ctx.size().unwrap_or(WidgetSize::Medium);
            log_msg(&format!("CreateWidget: id={}, size={:?}", id, size));

            if let Ok(mut st) = me.state.lock() {
                st.games = scan_installed_games();
                st.widgets.insert(id.clone(), WidgetInstance { size, page: 0 });
            }
            me.push_widget_update(&id, None);
        }
        S_OK
    }

    unsafe extern "system" fn delete_widget(
        this: *mut c_void,
        widget_id: *mut c_void,
        _custom_state: *mut c_void,
    ) -> HRESULT {
        let me = &*(this as *const Self);
        let id_h: &HSTRING = std::mem::transmute(&widget_id);
        let id = id_h.to_string_lossy();
        log_msg(&format!("DeleteWidget: id={}", id));

        if let Ok(mut st) = me.state.lock() {
            st.widgets.remove(&id);
        }
        S_OK
    }

    unsafe extern "system" fn on_action_invoked(
        this: *mut c_void,
        action_invoked_args: *mut c_void,
    ) -> HRESULT {
        let me = &*(this as *const Self);
        if action_invoked_args.is_null() {
            return S_OK;
        }
        if let Some(args) = IWidgetActionInvokedArgs::from_raw_borrowed(&action_invoked_args) {
            let verb = args.verb().unwrap_or_default();
            let data = args.data().unwrap_or_default();
            let widget_id = args
                .widget_context()
                .and_then(|c| c.id())
                .unwrap_or_default();
            log_msg(&format!(
                "OnActionInvoked: id={}, verb={}, data={}",
                widget_id, verb, data
            ));

            match verb.as_str() {
                "launch" => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(appid) = val.get("appid").and_then(|v| v.as_str()) {
                            let name = val
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("juego");
                            log_msg(&format!("Launching Steam game appid={}", appid));
                            launch_steam_game(appid);

                            let now_ts = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            if let Ok(mut st) = me.state.lock() {
                                if let Some(g) = st.games.iter_mut().find(|g| g.appid == appid) {
                                    g.last_played = now_ts.max(g.last_played + 1);
                                }
                                st.games.sort_by(|a, b| {
                                    b.last_played
                                        .cmp(&a.last_played)
                                        .then_with(|| b.playtime_minutes.cmp(&a.playtime_minutes))
                                        .then_with(|| {
                                            a.name.to_lowercase().cmp(&b.name.to_lowercase())
                                        })
                                });
                            }

                            let status = format!("Ejecutando {}...", name);
                            me.push_widget_update(&widget_id, Some(&status));
                            return S_OK;
                        }
                    }
                }
                "refresh" => {
                    if let Ok(mut st) = me.state.lock() {
                        st.games = scan_installed_games();
                    }
                    me.push_widget_update(&widget_id, None);
                }
                "open_steam" => {
                    open_steam_library();
                    me.push_widget_update(&widget_id, Some("Abriendo Steam..."));
                }
                "next_page" => {
                    if let Ok(mut st) = me.state.lock() {
                        if let Some(inst) = st.widgets.get_mut(&widget_id) {
                            inst.page = inst.page.saturating_add(1);
                        }
                    }
                    me.push_widget_update(&widget_id, None);
                }
                "prev_page" => {
                    if let Ok(mut st) = me.state.lock() {
                        if let Some(inst) = st.widgets.get_mut(&widget_id) {
                            inst.page = inst.page.saturating_sub(1);
                        }
                    }
                    me.push_widget_update(&widget_id, None);
                }
                _ => {}
            }
        }
        S_OK
    }

    unsafe extern "system" fn on_widget_context_changed(
        this: *mut c_void,
        context_changed_args: *mut c_void,
    ) -> HRESULT {
        let me = &*(this as *const Self);
        if context_changed_args.is_null() {
            return S_OK;
        }
        if let Some(args) = IWidgetContextChangedArgs::from_raw_borrowed(&context_changed_args) {
            if let Ok(ctx) = args.widget_context() {
                let id = ctx.id().unwrap_or_default();
                let size = ctx.size().unwrap_or(WidgetSize::Medium);
                log_msg(&format!("OnWidgetContextChanged: id={}, size={:?}", id, size));

                if let Ok(mut st) = me.state.lock() {
                    let entry = st
                        .widgets
                        .entry(id.clone())
                        .or_insert(WidgetInstance { size, page: 0 });
                    entry.size = size;
                }
                me.push_widget_update(&id, None);
            }
        }
        S_OK
    }

    unsafe extern "system" fn activate(
        this: *mut c_void,
        widget_context: *mut c_void,
    ) -> HRESULT {
        let me = &*(this as *const Self);
        if widget_context.is_null() {
            return S_OK;
        }
        if let Some(ctx) = IWidgetContext::from_raw_borrowed(&widget_context) {
            let id = ctx.id().unwrap_or_default();
            let size = ctx.size().unwrap_or(WidgetSize::Medium);
            log_msg(&format!("Activate: id={}, size={:?}", id, size));

            if let Ok(mut st) = me.state.lock() {
                st.games = scan_installed_games();
                let entry = st
                    .widgets
                    .entry(id.clone())
                    .or_insert(WidgetInstance { size, page: 0 });
                entry.size = size;
            }
            me.push_widget_update(&id, None);
        }
        S_OK
    }

    unsafe extern "system" fn deactivate(_this: *mut c_void, widget_id: *mut c_void) -> HRESULT {
        let id_h: &HSTRING = std::mem::transmute(&widget_id);
        let id = id_h.to_string_lossy();
        log_msg(&format!("Deactivate: id={}", id));
        S_OK
    }

    fn recover_existing_widgets(&self) {
        if let Ok(mgr) = get_widget_manager() {
            unsafe {
                let mut count: u32 = 0;
                let mut infos_ptr: *mut *mut c_void = std::ptr::null_mut();
                if (Interface::vtable(&mgr).GetWidgetInfos)(
                    Interface::as_raw(&mgr),
                    &mut count,
                    &mut infos_ptr,
                )
                .is_ok()
                    && !infos_ptr.is_null()
                {
                    let slice = std::slice::from_raw_parts(infos_ptr, count as usize);
                    for &raw_info in slice {
                        if !raw_info.is_null() {
                            let info = IWidgetInfo::from_raw(raw_info);
                            if let Ok(ctx) = info.widget_context() {
                                let id = ctx.id().unwrap_or_default();
                                let size = ctx.size().unwrap_or(WidgetSize::Medium);
                                let custom = info.custom_state().unwrap_or_default();
                                let parsed: WidgetCustomState =
                                    serde_json::from_str(&custom).unwrap_or_default();
                                if !id.is_empty() {
                                    if let Ok(mut st) = self.state.lock() {
                                        st.widgets.insert(
                                            id.clone(),
                                            WidgetInstance {
                                                size,
                                                page: parsed.page,
                                            },
                                        );
                                    }
                                    self.push_widget_update(&id, None);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn push_widget_update(&self, widget_id: &str, status_msg: Option<&str>) {
        let (games, size, page) = {
            let st = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let inst = st
                .widgets
                .get(widget_id)
                .cloned()
                .unwrap_or(WidgetInstance {
                    size: WidgetSize::Medium,
                    page: 0,
                });
            (st.games.clone(), inst.size, inst.page)
        };

        let (template_json, data_json) = build_adaptive_card(&games, page, size, status_msg);
        let custom_state = serde_json::to_string(&WidgetCustomState { page }).unwrap_or_default();

        match get_widget_manager() {
            Ok(mgr) => {
                match create_update_options(widget_id, &template_json, &data_json, &custom_state) {
                    Ok(opts) => unsafe {
                        let hr = (Interface::vtable(&mgr).UpdateWidget)(
                            Interface::as_raw(&mgr),
                            opts.as_raw(),
                        );
                        log_msg(&format!(
                            "UpdateWidget(id={}) -> HR=0x{:08X}",
                            widget_id, hr.0 as u32
                        ));
                    },
                    Err(e) => log_msg(&format!("create_update_options failed: {:?}", e)),
                }
            }
            Err(e) => log_msg(&format!("get_widget_manager failed: {:?}", e)),
        }
    }
}

#[implement(IClassFactory, Agile = false)]
pub struct WidgetProviderFactory {
    provider: *mut SteamWidgetProvider,
}

unsafe impl Send for WidgetProviderFactory {}
unsafe impl Sync for WidgetProviderFactory {}

impl WidgetProviderFactory {
    pub fn new() -> Self {
        Self {
            provider: SteamWidgetProvider::new_boxed(),
        }
    }
}

impl IClassFactory_Impl for WidgetProviderFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        unsafe {
            if ppvobject.is_null() {
                return Err(Error::from_hresult(E_POINTER));
            }
            *ppvobject = std::ptr::null_mut();

            if punkouter.is_some() {
                return Err(Error::from_hresult(CLASS_E_CLASSNOTAVAILABLE));
            }

            log_msg(&format!(
                "ClassFactory::CreateInstance querying IID {:?}",
                *riid
            ));
            let hr = SteamWidgetProvider::qi(self.provider as *mut c_void, riid, ppvobject);
            if hr.is_err() {
                log_msg(&format!(
                    "ClassFactory QueryInterface failed for IID {:?}",
                    *riid
                ));
                return Err(Error::from_hresult(hr));
            }
            log_msg("ClassFactory returned SteamWidgetProvider instance");
            Ok(())
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;
use windows_core::*;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WidgetSize {
    Small = 0,
    Medium = 1,
    Large = 2,
}

windows_core::imp::define_interface!(
    IWidgetContext,
    IWidgetContext_Vtbl,
    0x903c518b_40bc_5bc6_88f7_af9d81c0cdc1
);
windows_core::imp::interface_hierarchy!(IWidgetContext, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetContext_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub Id: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub DefinitionId: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Size: unsafe extern "system" fn(*mut c_void, *mut WidgetSize) -> HRESULT,
    pub IsActive: unsafe extern "system" fn(*mut c_void, *mut bool) -> HRESULT,
}

impl IWidgetContext {
    pub fn id(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).Id)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }

    pub fn definition_id(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).DefinitionId)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }

    pub fn size(&self) -> Result<WidgetSize> {
        unsafe {
            let mut sz = WidgetSize::Medium;
            (Interface::vtable(self).Size)(Interface::as_raw(self), &mut sz).ok()?;
            Ok(sz)
        }
    }

    pub fn is_active(&self) -> Result<bool> {
        unsafe {
            let mut active = false;
            (Interface::vtable(self).IsActive)(Interface::as_raw(self), &mut active).ok()?;
            Ok(active)
        }
    }
}

windows_core::imp::define_interface!(
    IWidgetActionInvokedArgs,
    IWidgetActionInvokedArgs_Vtbl,
    0xc593cc57_04b9_52ca_88ad_46fea21ea340
);
windows_core::imp::interface_hierarchy!(IWidgetActionInvokedArgs, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetActionInvokedArgs_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub WidgetContext: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Verb: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Data: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub CustomState: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

impl IWidgetActionInvokedArgs {
    pub fn widget_context(&self) -> Result<IWidgetContext> {
        unsafe {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).WidgetContext)(Interface::as_raw(self), &mut ptr).ok()?;
            if ptr.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
            }
            Ok(IWidgetContext::from_raw(ptr))
        }
    }

    pub fn verb(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).Verb)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }

    pub fn data(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).Data)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }

    pub fn custom_state(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).CustomState)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }
}

windows_core::imp::define_interface!(
    IWidgetContextChangedArgs,
    IWidgetContextChangedArgs_Vtbl,
    0x2c226d54_2252_576b_a197_370b28d25c2f
);
windows_core::imp::interface_hierarchy!(IWidgetContextChangedArgs, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetContextChangedArgs_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub WidgetContext: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

impl IWidgetContextChangedArgs {
    pub fn widget_context(&self) -> Result<IWidgetContext> {
        unsafe {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).WidgetContext)(Interface::as_raw(self), &mut ptr).ok()?;
            if ptr.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
            }
            Ok(IWidgetContext::from_raw(ptr))
        }
    }
}

windows_core::imp::define_interface!(
    IWidgetInfo,
    IWidgetInfo_Vtbl,
    0xcea11f42_a020_5db5_89e2_b7dece4ae5cb
);
windows_core::imp::interface_hierarchy!(IWidgetInfo, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetInfo_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub WidgetContext: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Template: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Data: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub CustomState: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub LastUpdateTime: unsafe extern "system" fn(*mut c_void, *mut i64) -> HRESULT,
}

impl IWidgetInfo {
    pub fn widget_context(&self) -> Result<IWidgetContext> {
        unsafe {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).WidgetContext)(Interface::as_raw(self), &mut ptr).ok()?;
            if ptr.is_null() {
                return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
            }
            Ok(IWidgetContext::from_raw(ptr))
        }
    }

    pub fn custom_state(&self) -> Result<String> {
        unsafe {
            let mut hstr: *mut c_void = std::ptr::null_mut();
            (Interface::vtable(self).CustomState)(Interface::as_raw(self), &mut hstr).ok()?;
            let s: HSTRING = std::mem::transmute(hstr);
            Ok(s.to_string_lossy())
        }
    }
}

windows_core::imp::define_interface!(
    IWidgetUpdateRequestOptions,
    IWidgetUpdateRequestOptions_Vtbl,
    0xb09ca8f7_7424_5687_baaf_7dd6fa639672
);
windows_core::imp::interface_hierarchy!(IWidgetUpdateRequestOptions, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetUpdateRequestOptions_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub WidgetId: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub Template: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub SetTemplate: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub Data: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub SetData: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub CustomState: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
    pub SetCustomState: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    IWidgetUpdateRequestOptionsFactory,
    IWidgetUpdateRequestOptionsFactory_Vtbl,
    0xe0e00af8_1d10_57a8_9419_3f568e854daa
);
windows_core::imp::interface_hierarchy!(IWidgetUpdateRequestOptionsFactory, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetUpdateRequestOptionsFactory_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub CreateInstance:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    IWidgetManager,
    IWidgetManager_Vtbl,
    0x71cb10c0_671e_48e3_b995_207940397123
);
windows_core::imp::interface_hierarchy!(IWidgetManager, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetManager_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub UpdateWidget: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub GetWidgetIds:
        unsafe extern "system" fn(*mut c_void, *mut u32, *mut *mut *mut c_void) -> HRESULT,
    pub GetWidgetInfo:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> HRESULT,
    pub GetWidgetInfos:
        unsafe extern "system" fn(*mut c_void, *mut u32, *mut *mut *mut c_void) -> HRESULT,
    pub DeleteWidget: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    IWidgetManagerStatics,
    IWidgetManagerStatics_Vtbl,
    0x7f233b06_28e5_5e2b_8c04_a4fa747c28c7
);
windows_core::imp::interface_hierarchy!(IWidgetManagerStatics, IUnknown, IInspectable);

#[repr(C)]
pub struct IWidgetManagerStatics_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub GetDefault: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

// IWidgetProvider (implemented by our COM Server)
windows_core::imp::define_interface!(
    IWidgetProvider,
    IWidgetProvider_Vtbl,
    0x5c5774cc_72a0_452d_b9ed_075c0dd25eed
);
windows_core::imp::interface_hierarchy!(IWidgetProvider, IUnknown, IInspectable);

impl RuntimeType for IWidgetProvider {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}

impl RuntimeName for IWidgetProvider {
    const NAME: &'static str = "Microsoft.Windows.Widgets.Providers.IWidgetProvider";
}

pub trait IWidgetProvider_Impl: IUnknownImpl {
    unsafe fn CreateWidget(&self, widget_context: *mut c_void) -> HRESULT;
    unsafe fn DeleteWidget(&self, widget_id: *mut c_void, custom_state: *mut c_void) -> HRESULT;
    unsafe fn OnActionInvoked(&self, action_invoked_args: *mut c_void) -> HRESULT;
    unsafe fn OnWidgetContextChanged(&self, context_changed_args: *mut c_void) -> HRESULT;
    unsafe fn Activate(&self, widget_context: *mut c_void) -> HRESULT;
    unsafe fn Deactivate(&self, widget_id: *mut c_void) -> HRESULT;
}

#[repr(C)]
pub struct IWidgetProvider_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub CreateWidget: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub DeleteWidget: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> HRESULT,
    pub OnActionInvoked: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub OnWidgetContextChanged: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub Activate: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    pub Deactivate: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
}

impl IWidgetProvider_Vtbl {
    pub const fn new<Identity: IWidgetProvider_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CreateWidget<
            Identity: IWidgetProvider_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            widget_context: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::CreateWidget(this, widget_context)
            }
        }
        unsafe extern "system" fn DeleteWidget<
            Identity: IWidgetProvider_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            widget_id: *mut c_void,
            custom_state: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::DeleteWidget(this, widget_id, custom_state)
            }
        }
        unsafe extern "system" fn OnActionInvoked<
            Identity: IWidgetProvider_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            action_invoked_args: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::OnActionInvoked(this, action_invoked_args)
            }
        }
        unsafe extern "system" fn OnWidgetContextChanged<
            Identity: IWidgetProvider_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            context_changed_args: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::OnWidgetContextChanged(this, context_changed_args)
            }
        }
        unsafe extern "system" fn Activate<Identity: IWidgetProvider_Impl, const OFFSET: isize>(
            this: *mut c_void,
            widget_context: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::Activate(this, widget_context)
            }
        }
        unsafe extern "system" fn Deactivate<
            Identity: IWidgetProvider_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            widget_id: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider_Impl::Deactivate(this, widget_id)
            }
        }
        Self {
            base__: IInspectable_Vtbl::new::<Identity, IWidgetProvider, OFFSET>(),
            CreateWidget: CreateWidget::<Identity, OFFSET>,
            DeleteWidget: DeleteWidget::<Identity, OFFSET>,
            OnActionInvoked: OnActionInvoked::<Identity, OFFSET>,
            OnWidgetContextChanged: OnWidgetContextChanged::<Identity, OFFSET>,
            Activate: Activate::<Identity, OFFSET>,
            Deactivate: Deactivate::<Identity, OFFSET>,
        }
    }

    pub fn matches(iid: &GUID) -> bool {
        iid == &<IWidgetProvider as Interface>::IID
    }
}

// IWidgetProvider2 (implemented by our COM Server)
windows_core::imp::define_interface!(
    IWidgetProvider2,
    IWidgetProvider2_Vtbl,
    0x38c3a963_dd93_479d_9276_04bf84ee1816
);
windows_core::imp::interface_hierarchy!(IWidgetProvider2, IUnknown, IInspectable);

impl RuntimeType for IWidgetProvider2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}

impl RuntimeName for IWidgetProvider2 {
    const NAME: &'static str = "Microsoft.Windows.Widgets.Providers.IWidgetProvider2";
}

pub trait IWidgetProvider2_Impl: IUnknownImpl {
    unsafe fn OnCustomizationRequested(&self, args: *mut c_void) -> HRESULT;
}

#[repr(C)]
pub struct IWidgetProvider2_Vtbl {
    pub base__: IInspectable_Vtbl,
    pub OnCustomizationRequested: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
}

impl IWidgetProvider2_Vtbl {
    pub const fn new<Identity: IWidgetProvider2_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn OnCustomizationRequested<
            Identity: IWidgetProvider2_Impl,
            const OFFSET: isize,
        >(
            this: *mut c_void,
            args: *mut c_void,
        ) -> HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWidgetProvider2_Impl::OnCustomizationRequested(this, args)
            }
        }
        Self {
            base__: IInspectable_Vtbl::new::<Identity, IWidgetProvider2, OFFSET>(),
            OnCustomizationRequested: OnCustomizationRequested::<Identity, OFFSET>,
        }
    }

    pub fn matches(iid: &GUID) -> bool {
        iid == &<IWidgetProvider2 as Interface>::IID
    }
}

pub fn get_widget_manager() -> Result<IWidgetManager> {
    use windows::Win32::System::WinRT::RoGetActivationFactory;
    unsafe {
        let class_name = HSTRING::from("Microsoft.Windows.Widgets.Providers.WidgetManager");
        let statics: IWidgetManagerStatics = RoGetActivationFactory(&class_name)?;
        let mut ptr: *mut c_void = std::ptr::null_mut();
        (Interface::vtable(&statics).GetDefault)(Interface::as_raw(&statics), &mut ptr).ok()?;
        if ptr.is_null() {
            return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
        }
        Ok(IWidgetManager::from_raw(ptr))
    }
}

pub fn create_update_options(
    widget_id: &str,
    template_json: &str,
    data_json: &str,
    custom_state: &str,
) -> Result<IWidgetUpdateRequestOptions> {
    use windows::Win32::System::WinRT::RoGetActivationFactory;
    unsafe {
        let class_name =
            HSTRING::from("Microsoft.Windows.Widgets.Providers.WidgetUpdateRequestOptions");
        let factory: IWidgetUpdateRequestOptionsFactory = RoGetActivationFactory(&class_name)?;

        let id_h = HSTRING::from(widget_id);
        let mut ptr: *mut c_void = std::ptr::null_mut();
        (Interface::vtable(&factory).CreateInstance)(
            Interface::as_raw(&factory),
            std::mem::transmute_copy(&id_h),
            &mut ptr,
        )
        .ok()?;
        if ptr.is_null() {
            return Err(Error::from_hresult(HRESULT(0x80004003u32 as i32)));
        }
        let opts = IWidgetUpdateRequestOptions::from_raw(ptr);

        let tmpl_h = HSTRING::from(template_json);
        (Interface::vtable(&opts).SetTemplate)(
            Interface::as_raw(&opts),
            std::mem::transmute_copy(&tmpl_h),
        )
        .ok()?;

        let data_h = HSTRING::from(data_json);
        (Interface::vtable(&opts).SetData)(
            Interface::as_raw(&opts),
            std::mem::transmute_copy(&data_h),
        )
        .ok()?;

        let state_h = HSTRING::from(custom_state);
        (Interface::vtable(&opts).SetCustomState)(
            Interface::as_raw(&opts),
            std::mem::transmute_copy(&state_h),
        )
        .ok()?;

        Ok(opts)
    }
}

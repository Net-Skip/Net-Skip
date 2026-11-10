// Copyright © The Daybrite Project
// SPDX-License-Identifier: MPL-2.0
//! Per-platform touches Day's portable pieces leave to the app (https://daybrite.dev/docs/reference/tweaks):
//! the address field drawn without its own frame, so the rounded bar around it is the one shape
//! the user sees, as in Safari and Chrome.

use day::prelude::*;

/// Native look adjustments for the address bar's text field.
pub trait AddressFieldLook: Decorate + Sized {
    /// Draw the field without its own bezel, border or background, leaving the caret and text.
    fn frameless(self) -> Decorated<Self> {
        #[cfg(feature = "appkit")]
        {
            use day_toolkit_appkit::ext::AppKitExt;
            self.appkit(|view, _class, _mtm| {
                use objc2::runtime::{AnyObject, Sel};
                use objc2::{msg_send, sel};
                let obj: &AnyObject = view;
                let responds = |s: Sel| -> bool {
                    // SAFETY: `respondsToSelector:` is defined on every NSObject.
                    unsafe { msg_send![obj, respondsToSelector: s] }
                };
                if responds(sel!(setBezeled:)) {
                    // SAFETY: the view answers these NSTextField setters (checked above); each
                    // takes a BOOL, and the view is used on the main thread the tweak runs on.
                    unsafe {
                        let _: () = msg_send![obj, setBezeled: false];
                        let _: () = msg_send![obj, setBordered: false];
                        let _: () = msg_send![obj, setDrawsBackground: false];
                        // NSFocusRingTypeNone: the bar around the field shows focus instead.
                        let _: () = msg_send![obj, setFocusRingType: 1isize];
                    }
                }
            })
        }
        #[cfg(feature = "uikit")]
        {
            use day_toolkit_uikit::ext::UiKitExt;
            self.uikit(|view, _class, _mtm| {
                use objc2::runtime::AnyObject;
                use objc2::{msg_send, sel};
                let obj: &AnyObject = view;
                // SAFETY: `respondsToSelector:` is defined on every NSObject.
                let field: bool =
                    unsafe { msg_send![obj, respondsToSelector: sel!(setBorderStyle:)] };
                if field {
                    // SAFETY: a UITextField (checked above); UITextBorderStyleNone is 0, and the
                    // background color may be nil. Main thread: tweaks run at mount.
                    unsafe {
                        let _: () = msg_send![obj, setBorderStyle: 0isize];
                        let nil: *const AnyObject = std::ptr::null();
                        let _: () = msg_send![obj, setBackgroundColor: nil];
                    }
                }
            })
        }
        #[cfg(feature = "gtk")]
        {
            use day_toolkit_gtk::ext::GtkExt;
            self.gtk(|widget, _class| {
                use gtk4::prelude::ObjectExt;
                if widget.has_property("has-frame") {
                    widget.set_property("has-frame", false);
                }
            })
        }
        #[cfg(feature = "material")]
        {
            use day_toolkit_material::ext::MaterialExt;
            self.android(|view, class, env| {
                if !class.ends_with("TextInputLayout") {
                    return;
                }
                let _ = (|| -> jni::errors::Result<()> {
                    // No outline and no floating label: the bar around the field is the shape,
                    // and with the label off Material shows the placeholder inside the field.
                    let hint = env
                        .call_method(
                            view,
                            jni::jni_str!("getHint"),
                            jni::jni_sig!("()Ljava/lang/CharSequence;"),
                            &[],
                        )?
                        .l()?;
                    env.call_method(
                        view,
                        jni::jni_str!("setBoxBackgroundMode"),
                        jni::jni_sig!("(I)V"),
                        &[jni::JValue::Int(0)],
                    )?;
                    env.call_method(
                        view,
                        jni::jni_str!("setHintEnabled"),
                        jni::jni_sig!("(Z)V"),
                        &[jni::JValue::Bool(false)],
                    )?;
                    env.call_method(
                        view,
                        jni::jni_str!("setHint"),
                        jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
                        &[jni::JValue::Object(&hint)],
                    )?;
                    let edit = env
                        .call_method(
                            view,
                            jni::jni_str!("getEditText"),
                            jni::jni_sig!("()Landroid/widget/EditText;"),
                            &[],
                        )?
                        .l()?;
                    if !edit.is_null() {
                        env.call_method(
                            &edit,
                            jni::jni_str!("setBackgroundColor"),
                            jni::jni_sig!("(I)V"),
                            &[jni::JValue::Int(0)],
                        )?;
                        env.call_method(
                            &edit,
                            jni::jni_str!("setPadding"),
                            jni::jni_sig!("(IIII)V"),
                            &[
                                jni::JValue::Int(0),
                                jni::JValue::Int(0),
                                jni::JValue::Int(0),
                                jni::JValue::Int(0),
                            ],
                        )?;
                    }
                    Ok(())
                })();
            })
        }
        #[cfg(not(any(
            feature = "appkit",
            feature = "uikit",
            feature = "gtk",
            feature = "material"
        )))]
        {
            self.tweak(|_| {})
        }
    }
}

impl<P: Decorate> AddressFieldLook for P {}

use leptos::prelude::*;

use super::{NavigationDiagnostic, PaneNavigation};
use crate::PaneData;

/// Opt into browser Back/Forward for one provider. The URL is left unchanged.
///
/// There can be one mounted owner per browser window. Null and plain-object
/// history state can be extended; other state is left intact with a diagnostic.
/// Diagnostics default to the browser console when no callback is supplied.
#[component]
pub fn PaneBrowserHistory<D: PaneData + Send + Sync>(
    navigation: PaneNavigation<D>,
    namespace: &'static str,
    #[prop(optional)] on_diagnostic: Option<Callback<NavigationDiagnostic>>,
) -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    wasm::install(navigation, namespace, on_diagnostic);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (navigation, namespace, on_diagnostic);
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::super::{project_navigation, NavigationDiagnostic, PaneNavigation, Projection};
    use crate::PaneData;
    use leptos::prelude::*;
    use send_wrapper::SendWrapper;
    use std::sync::Arc;
    use wasm_bindgen::{prelude::*, JsCast};

    // The envelope and global owner never escape this module. Object.assign
    // preserves host fields, and omitting a URL preserves path/query/hash.
    #[wasm_bindgen(inline_js = r#"
const ownerKey = Symbol.for('mullion.pane-history.owner');
const stateKey = '__mullionPaneNavigation';
export function mullionHistoryAcquire(namespace) {
    if (window[ownerKey]) throw new Error('Mullion browser history already has an owner');
    const owner = { namespace };
    window[ownerKey] = owner;
    return owner;
}
export function mullionHistoryRelease(owner) {
    if (window[ownerKey] === owner) delete window[ownerKey];
}
export function mullionHistoryRead(owner) {
    const state = window.history.state;
    const envelope = state && state[stateKey];
    if (!envelope || envelope.namespace !== owner.namespace) return undefined;
    if (envelope.version !== 1 || typeof envelope.projection !== 'string')
        throw new Error('Unsupported Mullion browser history envelope');
    return envelope.projection;
}
export function mullionHistoryWrite(owner, projection, push) {
    if (window[ownerKey] !== owner) throw new Error('Mullion browser history owner was disposed');
    const state = window.history.state;
    if (state !== null && (typeof state !== 'object' || Array.isArray(state) ||
        (Object.getPrototypeOf(state) !== Object.prototype && Object.getPrototypeOf(state) !== null)))
        throw new Error('Mullion cannot merge non-object browser history state');
    if (state && Object.prototype.hasOwnProperty.call(state, stateKey)) {
        const previous = state[stateKey];
        if (!previous || previous.namespace !== owner.namespace || previous.version !== 1)
            throw new Error('Mullion browser history namespace is already owned or incompatible');
    }
    const next = Object.assign({}, state);
    next[stateKey] = { version: 1, namespace: owner.namespace, projection };
    if (push) window.history.pushState(next, '');
    else window.history.replaceState(next, '');
}
"#)]
    extern "C" {
        #[wasm_bindgen(catch, js_name = mullionHistoryAcquire)]
        fn acquire(namespace: &str) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(js_name = mullionHistoryRelease)]
        fn release(owner: &JsValue);
        #[wasm_bindgen(catch, js_name = mullionHistoryRead)]
        fn read(owner: &JsValue) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(catch, js_name = mullionHistoryWrite)]
        fn write(owner: &JsValue, projection: &str, push: bool) -> Result<(), JsValue>;
    }

    fn report(callback: Option<Callback<NavigationDiagnostic>>, diagnostic: NavigationDiagnostic) {
        if let Some(callback) = callback {
            callback.run(diagnostic);
        } else {
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "Mullion navigation: {diagnostic:?}"
            )));
        }
    }

    fn browser_error(error: JsValue) -> NavigationDiagnostic {
        NavigationDiagnostic::Browser(format!("{error:?}"))
    }

    fn read_projection(owner: &JsValue) -> Result<Option<Projection>, NavigationDiagnostic> {
        read(owner)
            .map_err(browser_error)?
            .as_string()
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    NavigationDiagnostic::Browser(format!(
                        "Invalid Mullion history projection: {error}"
                    ))
                })
            })
            .transpose()
    }

    fn write_projection(
        owner: &JsValue,
        projection: &Projection,
        push: bool,
    ) -> Result<(), NavigationDiagnostic> {
        let json = serde_json::to_string(projection)
            .map_err(|error| NavigationDiagnostic::Browser(error.to_string()))?;
        write(owner, &json, push).map_err(browser_error)
    }

    pub(super) fn install<D: PaneData + Send + Sync>(
        navigation: PaneNavigation<D>,
        namespace: &str,
        callback: Option<Callback<NavigationDiagnostic>>,
    ) {
        if navigation
            .observer
            .with_value(|observer| observer.is_some())
        {
            report(
                callback,
                NavigationDiagnostic::Browser(
                    "This navigation controller already has a browser bridge".into(),
                ),
            );
            return;
        }
        let owner = match acquire(namespace) {
            Ok(owner) => SendWrapper::new(owner),
            Err(error) => {
                report(callback, browser_error(error));
                return;
            }
        };
        let initial = navigation.projection(None);
        let projection = match read_projection(&owner) {
            Ok(Some(saved)) => {
                match navigation.restore(&initial, &saved) {
                    Ok(diagnostics) => {
                        for diagnostic in diagnostics {
                            report(callback, diagnostic);
                        }
                    }
                    Err(result) => report(callback, NavigationDiagnostic::ReplayRejected(result)),
                }
                saved
            }
            Ok(None) => {
                if let Err(error) = write_projection(&owner, &initial, false) {
                    report(callback, error);
                    release(&owner);
                    return;
                }
                initial
            }
            Err(error) => {
                report(callback, error);
                release(&owner);
                return;
            }
        };
        let cursor = StoredValue::new(projection);
        let owner = StoredValue::new(owner);
        let nav_pop = navigation.clone();
        let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let next = owner.with_value(|owner| read_projection(owner));
            match next {
                Ok(Some(next)) => {
                    let previous = cursor.get_value();
                    match nav_pop.restore(&previous, &next) {
                        Ok(diagnostics) => {
                            for diagnostic in diagnostics {
                                report(callback, diagnostic);
                            }
                        }
                        Err(result) => {
                            report(callback, NavigationDiagnostic::ReplayRejected(result))
                        }
                    }
                    // Even rejected/conflicting entries become the browser cursor.
                    // Do not compensate with history.go or produce replay entries.
                    cursor.set_value(next);
                }
                Ok(None) => {} // Leave foreign history and external traversal alone.
                Err(error) => report(callback, error),
            }
        });
        let Some(window) = web_sys::window() else {
            owner.with_value(|owner| release(owner));
            return;
        };
        if let Err(error) =
            window.add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref())
        {
            report(callback, browser_error(error));
            owner.with_value(|owner| release(owner));
            return;
        }
        navigation
            .observer
            .set_value(Some(Arc::new(move |before, after| {
                let result = owner.with_value(|owner| {
                    let existing = read_projection(owner)?;
                    let baseline = existing.clone().unwrap_or_else(|| before.clone());
                    let (baseline, next) = project_navigation(baseline, &before, &after);
                    if existing.as_ref() != Some(&baseline) {
                        write_projection(owner, &baseline, false)?;
                    }
                    write_projection(owner, &next, true)?;
                    cursor.set_value(next);
                    Ok::<_, NavigationDiagnostic>(())
                });
                if let Err(error) = result {
                    report(callback, error);
                }
            })));
        let listener = SendWrapper::new(listener);
        let window = SendWrapper::new(window);
        on_cleanup(move || {
            let _ = window
                .remove_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref());
            navigation.observer.set_value(None);
            owner.with_value(|owner| release(owner));
        });
    }
}

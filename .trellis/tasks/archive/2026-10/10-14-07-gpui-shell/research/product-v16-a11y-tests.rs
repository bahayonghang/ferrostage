// Research harness: real Windows calls, no window creation or desktop mutation.
use crate::{util::client_top_left, window_handle::WindowHandle};
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::GetDesktopWindow};

#[test]
fn retired_hwnd_coordinate_query_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        // A permanently invalid native handle exercises the same Windows error
        // as the captured post-destruction ClientToScreen failure.
        let _ = client_top_left(WindowHandle(HWND(-1isize as *mut _)));
    });
    assert!(result.is_ok(), "retired HWND must return a provider error, not unwind");
}

#[test]
fn live_desktop_coordinate_query_still_succeeds() {
    let result = std::panic::catch_unwind(|| {
        let point = client_top_left(WindowHandle(unsafe { GetDesktopWindow() })).unwrap();
        assert!(point.x.is_finite() && point.y.is_finite());
    });
    assert!(result.is_ok());
}

#[test]
fn retired_hwnd_coordinate_query_returns_element_not_available() {
    let error = client_top_left(WindowHandle(HWND(-1isize as *mut _))).unwrap_err();
    assert_eq!(error.code(), crate::util::element_not_available().code());
}

#[test]
fn retained_context_com_queries_return_element_not_available() {
    use accesskit::{Node, NodeId, Rect, Role, TreeUpdate};
    use crate::context::{ActionHandlerNoMut, Context};
    use std::sync::Arc;
    use windows::Win32::UI::Accessibility::{
        IRawElementProviderFragment, IRawElementProviderFragmentRoot,
    };
    struct NoActions;
    impl ActionHandlerNoMut for NoActions {
        fn do_action(&self, _: accesskit::ActionRequest) {}
    }
    let id = NodeId(1);
    let mut node = Node::new(Role::Window);
    node.set_bounds(Rect::new(0.0, 0.0, 100.0, 100.0));
    let tree = accesskit_consumer::Tree::new(TreeUpdate {
        nodes: vec![(id, node)],
        tree: Some(accesskit::Tree::new(id)),
        tree_id: accesskit::TreeId::ROOT,
        focus: id,
    }, true);
    let root_id = tree.state().root_id();
    // Retain Context exactly as an already-upgraded/in-flight provider can.
    let context = Context::new(
        WindowHandle(HWND(-1isize as *mut _)), tree, Arc::new(NoActions), false,
    );
    let provider = context.get_or_create_platform_node(root_id);
    let fragment: IRawElementProviderFragment = provider.clone().into_interface();
    let root: IRawElementProviderFragmentRoot = provider.into_interface();
    assert_eq!(
        unsafe { fragment.BoundingRectangle() }.unwrap_err().code(),
        crate::util::element_not_available().code(),
    );
    assert_eq!(
        unsafe { root.ElementProviderFromPoint(10.0, 10.0) }.unwrap_err().code(),
        crate::util::element_not_available().code(),
    );
    assert_eq!(Arc::strong_count(&context), 1);
}

fn text_context(hwnd: HWND) -> std::sync::Arc<crate::context::Context> {
    text_context_with_bounds(hwnd, true)
}

fn text_context_with_bounds(hwnd: HWND, has_bounds: bool) -> std::sync::Arc<crate::context::Context> {
    use accesskit::{Node, NodeId, Rect, Role, TextDirection, TreeUpdate};
    use crate::context::{ActionHandlerNoMut, Context};
    use std::sync::Arc;
    struct NoActions;
    impl ActionHandlerNoMut for NoActions {
        fn do_action(&self, _: accesskit::ActionRequest) {}
    }
    let id = NodeId(1);
    let run_id = NodeId(2);
    let mut node = Node::new(Role::TextInput);
    if has_bounds {
        node.set_bounds(Rect::new(0.0, 0.0, 100.0, 100.0));
    }
    node.set_children(vec![run_id]);
    let mut run = Node::new(Role::TextRun);
    if has_bounds {
        run.set_bounds(Rect::new(10.0, 10.0, 26.0, 24.0));
    }
    run.set_value("AB");
    run.set_character_lengths([1, 1]);
    run.set_character_positions([0.0, 8.0]);
    run.set_character_widths([8.0, 8.0]);
    run.set_word_starts([0]);
    run.set_text_direction(TextDirection::LeftToRight);
    let tree = accesskit_consumer::Tree::new(TreeUpdate {
        nodes: vec![(id, node), (run_id, run)],
        tree: Some(accesskit::Tree::new(id)),
        tree_id: accesskit::TreeId::ROOT,
        focus: id,
    }, true);
    Context::new(WindowHandle(hwnd), tree, Arc::new(NoActions), false)
}

#[test]
fn retained_text_context_queries_return_element_not_available() {
    use windows::Win32::UI::Accessibility::{ITextProvider, UiaPoint};
    let context = text_context(HWND(-1isize as *mut _));
    let provider = context.get_or_create_platform_node(context.read_tree().state().root_id());
    let text: ITextProvider = provider.into_interface();
    let range = unsafe { text.DocumentRange() }.unwrap();
    assert_eq!(
        unsafe { text.RangeFromPoint(UiaPoint { x: 15.0, y: 15.0 }) }.unwrap_err().code(),
        crate::util::element_not_available().code(),
    );
    assert_eq!(
        unsafe { range.GetBoundingRectangles() }.unwrap_err().code(),
        crate::util::element_not_available().code(),
    );
}

#[test]
fn live_provider_all_four_coordinate_methods_preserve_values() {
    use windows::Win32::{
        System::Ole::{SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetUBound},
        UI::Accessibility::{IRawElementProviderFragment, IRawElementProviderFragmentRoot, ITextProvider, UiaPoint},
    };
    let hwnd = unsafe { GetDesktopWindow() };
    let origin = client_top_left(WindowHandle(hwnd)).unwrap();
    let context = text_context(hwnd);
    let provider = context.get_or_create_platform_node(context.read_tree().state().root_id());
    let fragment: IRawElementProviderFragment = provider.clone().into_interface();
    let root: IRawElementProviderFragmentRoot = provider.clone().into_interface();
    let text: ITextProvider = provider.into_interface();
    let bounds = unsafe { fragment.BoundingRectangle() }.unwrap();
    assert_eq!((bounds.left, bounds.top, bounds.width, bounds.height), (origin.x, origin.y, 100.0, 100.0));
    unsafe { root.ElementProviderFromPoint(origin.x + 15.0, origin.y + 15.0) }.unwrap();
    unsafe { text.RangeFromPoint(UiaPoint { x: origin.x + 15.0, y: origin.y + 15.0 }) }.unwrap();
    let range = unsafe { text.DocumentRange() }.unwrap();
    assert_eq!(unsafe { range.GetText(-1) }.unwrap().to_string(), "AB");
    let array = unsafe { range.GetBoundingRectangles() }.unwrap();
    assert!(!array.is_null());
    assert_eq!(unsafe { SafeArrayGetUBound(array, 1) }.unwrap(), 3);
    let mut values = [0.0f64; 4];
    for (index, value) in values.iter_mut().enumerate() {
        unsafe { SafeArrayGetElement(array, &(index as i32), std::ptr::from_mut(value).cast()) }.unwrap();
    }
    unsafe { SafeArrayDestroy(array) }.unwrap();
    assert_eq!(values, [origin.x + 10.0, origin.y + 10.0, 16.0, 14.0]);
}

#[test]
fn empty_geometry_with_retained_invalid_hwnd_keeps_upstream_results() {
    empty_geometry_keeps_upstream_results(HWND(-1isize as *mut _));
}

#[test]
fn empty_geometry_with_live_hwnd_keeps_upstream_results() {
    empty_geometry_keeps_upstream_results(unsafe { GetDesktopWindow() });
}

fn empty_geometry_keeps_upstream_results(hwnd: HWND) {
    use windows::Win32::UI::Accessibility::{IRawElementProviderFragment, ITextProvider};
    let context = text_context_with_bounds(hwnd, false);
    let provider = context.get_or_create_platform_node(context.read_tree().state().root_id());
    let fragment: IRawElementProviderFragment = provider.clone().into_interface();
    let text: ITextProvider = provider.into_interface();
    let bounds = unsafe { fragment.BoundingRectangle() }.unwrap();
    assert_eq!((bounds.left, bounds.top, bounds.width, bounds.height), (0.0, 0.0, 0.0, 0.0));
    let range = unsafe { text.DocumentRange() }.unwrap();
    assert_eq!(unsafe { range.GetText(-1) }.unwrap().to_string(), "AB");
    assert!(unsafe { range.GetBoundingRectangles() }.unwrap().is_null());
}

// Compiled by the implementer, executed only by Root's native acceptance owner.
#[cfg(feature = "root-native-window")]
#[test]
fn root_only_owned_hidden_hwnd_destroy_with_retained_context() {
    use std::{sync::{mpsc, Arc}, time::Duration};
    use windows::{core::w, Win32::UI::{
        Accessibility::{IRawElementProviderFragment, IRawElementProviderFragmentRoot, ITextProvider, UiaPoint},
        WindowsAndMessaging::{CreateWindowExW, DestroyWindow, IsWindow, WINDOW_EX_STYLE, WINDOW_STYLE},
    }};
    struct OwnedHiddenWindow(Option<HWND>);
    impl Drop for OwnedHiddenWindow {
        fn drop(&mut self) {
            if let Some(hwnd) = self.0.take() {
                unsafe { DestroyWindow(hwnd) }.unwrap();
            }
        }
    }
    for iteration in 0..16 {
        // Built-in STATIC class; no WS_VISIBLE, ShowWindow or activation call.
        let hwnd = unsafe { CreateWindowExW(
            WINDOW_EX_STYLE::default(), w!("STATIC"), w!("FerroStage F14 research"),
            WINDOW_STYLE::default(), 0, 0, 100, 100, None, None, None, None,
        ) }.unwrap();
        let mut owner = OwnedHiddenWindow(Some(hwnd));
        assert!(unsafe { IsWindow(Some(hwnd)) }.as_bool());
        client_top_left(WindowHandle(hwnd)).unwrap();
        let context = text_context(hwnd);
        let retained = Arc::clone(&context);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (go_tx, go_rx) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            let provider = retained.get_or_create_platform_node(retained.read_tree().state().root_id());
            let fragment: IRawElementProviderFragment = provider.clone().into_interface();
            let root: IRawElementProviderFragmentRoot = provider.clone().into_interface();
            let text: ITextProvider = provider.into_interface();
            let range = unsafe { text.DocumentRange() }.unwrap();
            ready_tx.send(()).unwrap();
            go_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let unavailable = crate::util::element_not_available().code();
            assert_eq!(unsafe { fragment.BoundingRectangle() }.unwrap_err().code(), unavailable);
            assert_eq!(unsafe { root.ElementProviderFromPoint(15.0, 15.0) }.unwrap_err().code(), unavailable);
            assert_eq!(unsafe { text.RangeFromPoint(UiaPoint { x: 15.0, y: 15.0 }) }.unwrap_err().code(), unavailable);
            assert_eq!(unsafe { range.GetBoundingRectangles() }.unwrap_err().code(), unavailable);
        });
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        drop(context);
        unsafe { DestroyWindow(owner.0.take().unwrap()) }.unwrap();
        assert!(!unsafe { IsWindow(Some(hwnd)) }.as_bool());
        go_tx.send(()).unwrap();
        worker.join().unwrap();
        println!("hidden owned HWND cycle {iteration}: native destroy then four retained-context COM errors returned normally");
    }
}

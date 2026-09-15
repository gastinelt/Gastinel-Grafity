//! Backend Win32 natif pour Gastinel Desktop.
//!
//! Fournit une fenêtre native Windows via FFI direct (user32.dll, kernel32.dll)
//! sans aucune dépendance externe. Compatible 100% offline.
//!
//! Traduit les messages Win32 (WM_*) en événements Gastinel pour le compositeur.

#![allow(non_snake_case, non_upper_case_globals)]

use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// ─── Types Win32 FFI ─────────────────────────────────────────────────────────

type HWND = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;
type HICON = *mut std::ffi::c_void;
type HCURSOR = *mut std::ffi::c_void;
type HBRUSH = *mut std::ffi::c_void;
type HMENU = *mut std::ffi::c_void;
type LPVOID = *mut std::ffi::c_void;
type LRESULT = isize;
type WPARAM = usize;
type LPARAM = isize;
type ATOM = u16;
type BOOL = i32;
type UINT = u32;
type DWORD = u32;
type LONG = i32;
type WORD = u16;

// ─── Constantes Win32 ────────────────────────────────────────────────────────

const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF0000;
const WS_VISIBLE: DWORD = 0x10000000;
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
const CS_HREDRAW: UINT = 0x0002;
const CS_VREDRAW: UINT = 0x0001;
const IDC_ARROW: *const u16 = 32512 as *const u16;
const COLOR_WINDOW: i32 = 5;
const SW_SHOW: i32 = 5;

const WM_DESTROY: UINT = 0x0002;
const WM_CLOSE: UINT = 0x0010;
const WM_SIZE: UINT = 0x0005;
const WM_PAINT: UINT = 0x000F;
const WM_MOUSEMOVE: UINT = 0x0200;
const WM_LBUTTONDOWN: UINT = 0x0201;
const WM_LBUTTONUP: UINT = 0x0202;
const WM_RBUTTONDOWN: UINT = 0x0204;
const WM_RBUTTONUP: UINT = 0x0205;
const WM_KEYDOWN: UINT = 0x0100;
const WM_KEYUP: UINT = 0x0101;
const WM_QUIT: UINT = 0x0012;

const PM_REMOVE: UINT = 0x0001;

// ─── Structures Win32 ────────────────────────────────────────────────────────

#[repr(C)]
struct WNDCLASSEXW {
    cbSize: UINT,
    style: UINT,
    lpfnWndProc: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: HICON,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct POINT {
    x: LONG,
    y: LONG,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINT,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: *mut std::ffi::c_void,
    fErase: BOOL,
    rcPaint: RECT,
    fRestore: BOOL,
    fIncUpdate: BOOL,
    rgbReserved: [u8; 32],
}

// ─── Imports FFI Win32 ───────────────────────────────────────────────────────

#[link(name = "user32")]
#[link(name = "gdi32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        x: i32, y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    fn UpdateWindow(hWnd: HWND) -> BOOL;
    fn PeekMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgMin: UINT, wMsgMax: UINT, wRemoveMsg: UINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PostQuitMessage(nExitCode: i32);
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> *mut std::ffi::c_void;
    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn DestroyWindow(hWnd: HWND) -> BOOL;
    fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
}

// ─── Événements Gastinel locaux ──────────────────────────────────────────────

/// Événement traduit depuis un message Win32.
#[derive(Clone, Debug, PartialEq)]
pub enum Win32Event {
    /// Mouvement du pointeur (coordonnées client).
    MouseMove { x: i32, y: i32 },
    /// Bouton gauche enfoncé.
    MouseLeftDown { x: i32, y: i32 },
    /// Bouton gauche relâché.
    MouseLeftUp { x: i32, y: i32 },
    /// Bouton droit enfoncé.
    MouseRightDown { x: i32, y: i32 },
    /// Bouton droit relâché.
    MouseRightUp { x: i32, y: i32 },
    /// Touche enfoncée (code virtuel Win32).
    KeyDown { vk_code: u32 },
    /// Touche relâchée.
    KeyUp { vk_code: u32 },
    /// La fenêtre a été redimensionnée.
    Resized { width: u32, height: u32 },
    /// Demande de rafraîchissement (WM_PAINT).
    Paint,
    /// Fenêtre fermée.
    Closed,
}

// ─── Fenêtre native Win32 ────────────────────────────────────────────────────

/// Fenêtre Win32 native pour Gastinel Desktop.
///
/// Crée une fenêtre réelle via l'API Win32, avec une boucle de messages
/// non-bloquante (PeekMessage) pour l'intégration dans la boucle de rendu.
pub struct Win32Window {
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
    pub is_open: bool,
    events: Vec<Win32Event>,
}

/// Extraction des coordonnées x,y depuis lParam.
#[inline]
fn loword(l: LPARAM) -> i32 {
    (l as WORD) as i16 as i32
}

#[inline]
fn hiword(l: LPARAM) -> i32 {
    ((l >> 16) as WORD) as i16 as i32
}

/// Callback principal de la fenêtre Win32.
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_PAINT => {
            let mut ps = std::mem::zeroed::<PAINTSTRUCT>();
            BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &ps);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

/// Encode une chaîne Rust en UTF-16 null-terminated pour Win32.
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

impl Win32Window {
    /// Crée et affiche une nouvelle fenêtre Win32 native.
    ///
    /// # Panics
    /// Panique si l'enregistrement de la classe ou la création de la fenêtre échoue.
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        unsafe {
            let h_instance = GetModuleHandleW(ptr::null());
            let class_name = to_wide("GastinelDesktopClass");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: ptr::null_mut(),
                hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: ptr::null_mut(),
            };

            let atom = RegisterClassExW(&wc);
            assert!(atom != 0, "Échec de RegisterClassExW pour Gastinel");

            let window_title = to_wide(title);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT, CW_USEDEFAULT,
                width as i32, height as i32,
                ptr::null_mut(),
                ptr::null_mut(),
                h_instance,
                ptr::null_mut(),
            );

            assert!(!hwnd.is_null(), "Échec de CreateWindowExW pour Gastinel");

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            println!("[WIN32] Fenêtre native créée: '{}' ({}x{})", title, width, height);

            Self {
                hwnd,
                width,
                height,
                is_open: true,
                events: Vec::with_capacity(64),
            }
        }
    }

    /// Pompe les messages Win32 et les traduit en événements Gastinel.
    ///
    /// Utilise PeekMessage (non-bloquant) pour s'intégrer dans la boucle de rendu
    /// du compositeur sans bloquer le thread principal.
    pub fn pump_events(&mut self) {
        self.events.clear();

        unsafe {
            let mut msg: MSG = std::mem::zeroed();

            while PeekMessageW(&mut msg, self.hwnd, 0, 0, PM_REMOVE) != 0 {
                match msg.message {
                    WM_QUIT => {
                        self.is_open = false;
                        self.events.push(Win32Event::Closed);
                        return;
                    }
                    WM_MOUSEMOVE => {
                        self.events.push(Win32Event::MouseMove {
                            x: loword(msg.lParam),
                            y: hiword(msg.lParam),
                        });
                    }
                    WM_LBUTTONDOWN => {
                        self.events.push(Win32Event::MouseLeftDown {
                            x: loword(msg.lParam),
                            y: hiword(msg.lParam),
                        });
                    }
                    WM_LBUTTONUP => {
                        self.events.push(Win32Event::MouseLeftUp {
                            x: loword(msg.lParam),
                            y: hiword(msg.lParam),
                        });
                    }
                    WM_RBUTTONDOWN => {
                        self.events.push(Win32Event::MouseRightDown {
                            x: loword(msg.lParam),
                            y: hiword(msg.lParam),
                        });
                    }
                    WM_RBUTTONUP => {
                        self.events.push(Win32Event::MouseRightUp {
                            x: loword(msg.lParam),
                            y: hiword(msg.lParam),
                        });
                    }
                    WM_KEYDOWN => {
                        self.events.push(Win32Event::KeyDown {
                            vk_code: msg.wParam as u32,
                        });
                    }
                    WM_KEYUP => {
                        self.events.push(Win32Event::KeyUp {
                            vk_code: msg.wParam as u32,
                        });
                    }
                    WM_SIZE => {
                        let w = loword(msg.lParam) as u32;
                        let h = hiword(msg.lParam) as u32;
                        self.width = w;
                        self.height = h;
                        self.events.push(Win32Event::Resized { width: w, height: h });
                    }
                    WM_PAINT => {
                        self.events.push(Win32Event::Paint);
                    }
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        }
    }

    /// Retourne les événements collectés lors du dernier pump_events().
    pub fn events(&self) -> &[Win32Event] {
        &self.events
    }

    /// Demande un rafraîchissement de la fenêtre (invalide la zone client).
    pub fn request_redraw(&self) {
        unsafe {
            InvalidateRect(self.hwnd, ptr::null(), 0);
        }
    }

    /// Retourne les dimensions actuelles de la zone client.
    pub fn client_size(&self) -> (u32, u32) {
        unsafe {
            let mut rc: RECT = std::mem::zeroed();
            GetClientRect(self.hwnd, &mut rc);
            ((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32)
        }
    }
}

impl Drop for Win32Window {
    fn drop(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                DestroyWindow(self.hwnd);
            }
            println!("[WIN32] Fenêtre détruite proprement");
        }
    }
}

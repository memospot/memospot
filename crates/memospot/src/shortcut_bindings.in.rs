shortcut_bindings! {
    TOGGLE_MENU_BAR => {
        command: "toggle_menu_bar",
        accelerator: "CmdOrCtrl+H",
        codes: ["KeyH"],
    },
    OPEN_SETTINGS => {
        command: "open_settings",
        accelerator: "CmdOrCtrl+,",
        codes: ["Comma"],
    },
    ZOOM_IN => {
        command: "zoom_in",
        accelerator: "CmdOrCtrl+=",
        codes: ["Equal", "NumpadAdd", "NumpadEqual"],
    },
    ZOOM_OUT => {
        command: "zoom_out",
        accelerator: "CmdOrCtrl+-",
        codes: ["Minus", "NumpadSubtract"],
    },
    RESET_ZOOM => {
        command: "reset_zoom",
        accelerator: "CmdOrCtrl+0",
        codes: ["Digit0", "Numpad0"],
    },
}

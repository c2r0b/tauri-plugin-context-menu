import * as tauriApi from 'https://esm.run/@tauri-apps/api';

document.getElementById('firstBox').addEventListener('contextmenu', (e) => {
    e.preventDefault();
    
    tauriApi.invoke('plugin:context_menu|show_context_menu', {
        theme: 'light',
        items: [
            {
                label: "This is a menu item",
                disabled: false,
            }
        ]
    });
});

document.getElementById('secondBox').addEventListener('contextmenu', (e) => {
    e.preventDefault();
    
    tauriApi.invoke('plugin:context_menu|show_context_menu', {
        theme: 'dark',
        items: [
            {
                label: "This is ANOTHER menu item",
                disabled: true
            }
        ]
    });
});
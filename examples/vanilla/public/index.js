import * as tauriApi from 'https://esm.run/@tauri-apps/api';
import * as tauriEvent from 'https://esm.run/@tauri-apps/api/event';
import * as tauriApiPath from 'https://esm.run/@tauri-apps/api/path';

async function registerListeners() {
    // on context menu item click
    await tauriEvent.listen('my_first_item', (event) => {
        alert(event.event);
    });

    // on context menu item click
    await tauriEvent.listen('my_second_item', (event) => {
        alert(event.event);
    });

    // on context menu item click
    await tauriEvent.listen('my_first_subitem', (event) => {
        alert(event.event);
    });

    // on context menu item closed
    await tauriEvent.listen('menu-did-close', (event) => {
        alert(event.event);
    });
}
registerListeners(); // Register event listeners once

window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    
    // get icon path
    (tauriApiPath.resolveResource('assets/16x16.png')).then((assetUrl) => {
        assetUrl = assetUrl.replace("\\\\?\\", "");

        // show context menu
        tauriApi.invoke('plugin:context_menu|show_context_menu', {
            pos: {
                x: e.clientX,
                y: e.clientY
            },
            items: [
                {
                    label: "My first item",
                    disabled: false,
                    event: "my_first_item",
                    shortcut: "alt+m",
                    icon_path: assetUrl
                },
                {
                    is_separator: true
                },
                {
                    label: "My second item",
                    disabled: false,
                    event: "my_second_item",
                    shortcut: "cmd+C"
                },
                {
                    label: "My third item",
                    disabled: false,
                    subitems: [
                        {
                            label: "My first subitem",
                            event: "my_first_subitem",
                            shortcut: "ctrl+m"
                        },
                        {
                            label: "My second subitem",
                            disabled: true
                        }
                    ]
                }
            ]
        });
    });
});
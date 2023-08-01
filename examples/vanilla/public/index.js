import * as tauriApi from 'https://esm.run/@tauri-apps/api';
import * as tauriEvent from 'https://esm.run/@tauri-apps/api/event';
import * as tauriApiPath from 'https://esm.run/@tauri-apps/api/path';

window.addEventListener('contextmenu', async (e) => {
    e.preventDefault();
    
    const assetUrl = await tauriApiPath.resolveResource('icons/32x32.png');

    tauriApi.invoke('plugin:context_menu|show_context_menu', {
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
                disabled: true,
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

    // on context menu item click
    const unlisten = await tauriEvent.listen('my_first_item', (event) => {
        unlisten();
        alert(event.event);
    });

    // on context menu item click
    const unlisten2 = await tauriEvent.listen('my_second_item', (event) => {
        unlisten2();
        alert(event.event);
    });

    // on context menu item click
    const unlisten3 = await tauriEvent.listen('menu-did-close', (event) => {
        unlisten3();
        alert(event.event);
    });

    // on context menu item click
    const unlisten4 = await tauriEvent.listen('my_first_subitem', (event) => {
        unlisten4();
        alert(event.event);
    });
});
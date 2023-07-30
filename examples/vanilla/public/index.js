import * as tauriApi from 'https://esm.run/@tauri-apps/api';
import * as tauriEvent from 'https://esm.run/@tauri-apps/api/event';

window.addEventListener('contextmenu', async () => {
    tauriApi.invoke('plugin:context_menu|show_context_menu', {
    items: [
        {
            label: "My first item",
            disabled: false,
            event_name: "my_first_item",
            key_binding: "Ctrl+M",
            hotkey: "Ctrl+M",
            icon_path: "assets/icons/16x16.png"
        },
        {
            label: "My second item",
            disabled: false,
            event_name: "my_second_item",
            key_binding: "Shift+C",
            hotkey: "Shift+C"
        },
        {
            label: "My third item",
            disabled: false,
            subitems: [
                {
                    label: "My first subitem",
                    disabled: false,
                    event_name: "my_first_subitem",
                    key_binding: "Ctrl+M",
                    hotkey: "Ctrl+M"
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
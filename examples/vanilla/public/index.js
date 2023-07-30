import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/event';

document.addEventListener('load', () => {
    console.log('loaded')
    window.addEventListener('contextmenu', async () => {
        invoke('plugin:context_menu|show_context_menu', {
        items: [
            {
                label: "My first item",
                disabled: false,
                event_name: "my_first_item",
                key_binding: "Ctrl+Shift+M",
                hotkey: "option+Shift+M",
                icon_path: "assets/icons/16x16.png"
            },
            {
                label: "My second item",
                disabled: false,
                event_name: "my_second_item",
                key_binding: "Ctrl+Shift+M",
                hotkey: "Ctrl+Shift+M"
            },
            {
                label: "My third item",
                disabled: false,
                subitems: [
                    {
                        label: "My first subitem",
                        disabled: false,
                        event_name: "my_first_subitem",
                        key_binding: "Ctrl+Shift+M",
                        hotkey: "Ctrl+Shift+M"
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
        const unlisten = await listen('my_first_item', (event) => {
        unlisten();
        alert(event.event);
        });
    
        // on context menu item click
        const unlisten2 = await listen('my_second_item', (event) => {
        unlisten2();
        alert(event.event);
        });
    
        // on context menu item click
        const unlisten3 = await listen('menu-did-close', (event) => {
        unlisten3();
        alert(event.event);
        });
    
        // on context menu item click
        const unlisten4 = await listen('my_first_subitem', (event) => {
        unlisten4();
        alert(event.event);
        });
    });
});
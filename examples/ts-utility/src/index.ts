import { assetToPath, onEventShowMenu } from 'tauri-plugin-context-menu';

onEventShowMenu('contextmenu', async (_e:MouseEvent) => {
    const options = {
        items: [
            {
                label: "My first item",
                disabled: false,
                event: (e:any) => {
                    alert(e.payload?.message);
                },
                payload: { message: "Hello from the payload!" },
                shortcut: "alt+m",
                icon: {
                    path: await assetToPath('assets/16x16.png'),
                    width: 32,
                    height: 32
                }
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
                        checked: true,
                        event: () => {
                            alert('My first subitem clicked');
                        },
                        shortcut: "ctrl+m"
                    },
                    {
                        label: "My second subitem",
                        disabled: true
                    }
                ]
            }
        ]
    };
    return options;
 });
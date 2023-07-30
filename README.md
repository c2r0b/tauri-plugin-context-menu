# Tauri Plugin Context Menu

A Tauri plugin to display native context menu.
The default Tauri API does not support context menu, so this plugin is created to fill the gap.

![Screenshot](./assets/screenshot.png)

## Support
All non-supported listed features are intended as future development.
|                  | MacOS   | Windows | Linux   |
| ---------------- | ------- | ------- | ------- |
| Usability        | ✅      | ❌       | ❌        |
| Submenu          | ✅      | ❌       | ❌        |
| Disabled         | ✅      | ❌       | ❌        |
| Callback         | ✅      | ❌       | ❌        |
| Shortcut         | ✅      | ❌       | ❌        |
| Icon             | 🔜      | ❌       | ❌        |
| Separator        | 🔜      | ❌       | ❌        |

## Installation
TODO  
See ["Using a Plugin" Tauri officiale guide](https://tauri.app/v1/guides/features/plugin#using-a-plugin) to initialize the plugin.

## Run Example
A vanilla JS example is provided in `examples/vanilla`. To run the example, run the following commands:

```bash
cd examples/svelte-app
npm run tauri dev
```

## Sample Usage

```ts
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

window.addEventListener("contextmenu", (e) => {
    e.preventDefault();

    // Show the context menu
    invoke("plugin:context_menu|show_context_menu", {
        items: [
            {
                label: "Item 1",
                disabled: false,
                event: "item1clicked",
                shortcut: "ctrl+M"
                subitems: [
                    {
                        label: "Subitem 1",
                        disabled: false,
                        event: "subitem1clicked",
                    },
                    {
                        label: "Subitem 2",
                        disabled: false,
                        event: "subitem2clicked",
                    }
                ]
            }
        ],
    });

    // Listen to the event emitted when the first menu item is clicked
    listen("item1clicked", () => {
        alert("item 1 clicked");
    });
});
```

## Options
List of options that can be passed to the plugin.
| Option | Type | Description |
| ------ | ---- | ----------- |
| items | `MenuItem[]` | List of menu items to be displayed. |
| pos | `Position` | Position of the menu. Default to the cursor position. |

### MenuItem
| Option | Type | Description |
| ------ | ---- | ----------- |
| label | `string` | Displayed test of the menu item. |
| disabled | `boolean` | Whether the menu item is disabled. |
| event | `string` | Event name to be emitted when the menu item is clicked. |
| subitems | `MenuItem[]` | List of sub menu items to be displayed. |
| shortcut | `string` | Keyboard shortcut displayed on the right. |

### Position
Position oordinates are relative to the currently active window.
| Option | Type | Description |
| ------ | ---- | ----------- |
| x | `number` | X position of the menu. |
| y | `number` | Y position of the menu. |
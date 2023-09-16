# Tauri Plugin Context Menu

A Tauri plugin to display native context menu on Tauri v1.x.
The Tauri API does not support native context menu out of the box, so this plugin is created to fill the gap.

![Screenshot](./assets/screenshot.png)

Official context menu support has been added in Tauri v2.x (see [here](https://github.com/tauri-apps/tauri/issues/4338)), so this plugin is intended to be used with Tauri v1.x only.

## Support
| Windows | MacOS | Linux (gtk) |
| ------- | ----- | ------- |
| ✅      | ✅   | ✅      |

## Installation
Crate: https://crates.io/crates/tauri-plugin-context-menu

`cargo add tauri-plugin-context-menu` to add the package.

Or add the following to your `Cargo.toml` for the latest unpublished version (not recommanded).

```toml
tauri-plugin-context-menu = { git = "https://github.com/c2r0b/tauri-plugin-context-menu", branch = "main" }
```

See ["Using a Plugin" Tauri official guide](https://tauri.app/v1/guides/features/plugin#using-a-plugin) to initialize the plugin.

## Run Example
A vanilla JS example is provided in `examples/vanilla`. To run the example, run the following commands:

```bash
cd examples/vanilla
npm run tauri dev
```

## Sample Usage

```ts
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { resolveResource } from "@tauri-apps/api/path";

// Listen to the event emitted when the first menu item is clicked
listen("item1clicked", () => {
    alert("item 1 clicked");
});

window.addEventListener("contextmenu", async (e) => {
    e.preventDefault();
    const iconUrl = await resolveResource('assets/16x16.png');

    // Show the context menu
    invoke("plugin:context_menu|show_context_menu", {
        items: [
            {
                label: "Item 1",
                disabled: false,
                event: "item1clicked",
                shortcut: "ctrl+M",
                icon: {
                    path: iconUrl
                },
                subitems: [
                    {
                        label: "Subitem 1",
                        disabled: true,
                        event: "subitem1clicked",
                    },
                    {
                        is_separator: true,
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
});
```

## Options
List of options that can be passed to the plugin.
| Option | Type | Description |
| ------ | ---- | ----------- |
| items | `MenuItem[]` | List of menu items to be displayed. |
| pos | `Position` | Position of the menu. Defaults to the cursor position. |

### MenuItem
| Option | Type | Optional | Default | Description |
| ------ | ---- |---- |---- | ----------- |
| label | `string` | | | Displayed test of the menu item. |
| disabled | `boolean` | `optional` |  `false` | Whether the menu item is disabled. |
| event | `string` | `optional` | | Event name to be emitted when the menu item is clicked. |
| subitems | `MenuItem[]` | `optional` |  `[]` | List of sub menu items to be displayed. |
| shortcut | `string` | `optional` | | Keyboard shortcut displayed on the right. |
| icon | `MenuItemIcon` | `optional` | | Icon to be displayed on the left. |
| is_separator | `boolean` | `optional` | `false` | Whether the menu item is a separator. |


### MenuItemIcon
| Option | Type | Optional | Default | Description |
| ------ | ---- |---- |---- | ----------- |
| path | `string` | | | Absolute path to the icon file. |
| width | `number` | `optional` | `16` | Width of the icon. |
| height | `number` | `optional` | `16` | Height of the icon. |

### Position
Position coordinates must be relative to the currently active window when `is_absolute` is set to `false`.
| Option | Type | Optional | Default | Description |
| ------ | ---- |---- |---- | ----------- |
| x | `number` | | | X position of the menu. |
| y | `number` | | | Y position of the menu. |
| is_absolute | `boolean` |`optional` | `false` |  Is the position absolute to the screen. |

## Events
### Item Clicked
Emitted when a menu item is clicked. The event name is the same as the `event` option of the menu item:

```ts
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api";

listen("[EVENTNAME]", () => {
    alert("menu item clicked");
});

invoke(...{
    items: [{
        ...
        event: "[EVENTNAME]",
        ...
    }]
});
```

### Menu Did Close
Emitted when the menu is closed. This event is emitted regardless of whether the menu is closed by clicking on a menu item or by clicking outside the menu.  
You can catch this event using the following code:

```ts
import { listen } from "@tauri-apps/api/event";

listen("menu-did-close", () => {
    alert("menu closed");
});
```
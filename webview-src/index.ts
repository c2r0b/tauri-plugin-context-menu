import * as tauriApi from '@tauri-apps/api';
import * as tauriEvent from '@tauri-apps/api/event';
import * as tauriApiPath from '@tauri-apps/api/path';

const SHOW_COMMAND = 'plugin:context_menu|show_context_menu';

import * as ContextMenu from './types';
export { ContextMenu };

export async function assetToPath(asset: string): Promise<string> {
    return await tauriApiPath.resolveResource(asset);
}

export function showMenu(options: ContextMenu.Options): void {
    // for each item, if it is a function, replace it with an event listener
    function processItems(items: ContextMenu.Item[], prefix: string): void {
      for (let i = 0; i < items.length; i++) {
        const itemEvent = items[i].event;
  
        if (typeof itemEvent === 'function') {
          const eventName = `${prefix}_context_menu_item_${i}`;
          
          // Listen to the event and call the function directly
          tauriEvent.listen(eventName, (e) => itemEvent(e));
          items[i].event = eventName;
        }
  
        // Recurse into subitems if they exist
        if (items[i].subitems) {
          processItems(items[i].subitems as ContextMenu.Item[], `${prefix}_${i}`);
        }
      }
    }
  
    processItems(options.items, 'root');
    
    // send the options to the plugin
    tauriApi.invoke(SHOW_COMMAND, options as any);
  }
  

export function onEventShowMenu(eventName: string, options: ContextMenu.EventOptions): void {
    window.addEventListener(eventName, async (e) => {
        e.preventDefault();

        // if options is a function, call it to get the options
        if (typeof options === 'function') {
            options = await options(e as MouseEvent);
        }

        showMenu(options);
    });
}
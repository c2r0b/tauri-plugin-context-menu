import * as tauriApi from '@tauri-apps/api';
import * as tauriEvent from '@tauri-apps/api/event';
import * as tauriApiPath from '@tauri-apps/api/path';

const SHOW_COMMAND = 'plugin:context_menu|show_context_menu';

import * as ContextMenu from './types';
export { ContextMenu };

export async function assetToPath(asset: string): Promise<string> {
	return await tauriApiPath.resolveResource(asset);
}

// for each item, if it is a function, replace it with an event listener
async function processItems(items: ContextMenu.Item[], prefix: string): Promise<ContextMenu.ProcessResult> {
	const unlisteners: tauriEvent.UnlistenFn[] = [];
	const processed:ContextMenu.Item[] = [ ...items.map((item) => ({ ...item })) ];

	for (let i = 0; i < processed.length; i++) {
		const itemEvent = processed[i].event;

		if (typeof itemEvent === 'function') {
			const eventName = `${prefix}_context_menu_item_${i}`;

			// Listen to the event and call the function directly
			unlisteners.push(await tauriEvent.listen(eventName, (e) => itemEvent(e)));
			processed[i].event = eventName;
		}

		// Recurse into subitems if they exist
		if (items[i].subitems) {
			const result = await processItems(items[i].subitems as ContextMenu.Item[], `${prefix}_${i}`);
			unlisteners.push(...result.unlisteners);
			processed[i].subitems = result.processed;
		}
	}

	return { unlisteners, processed };
}

export async function showMenu(options: ContextMenu.Options) {
	const { unlisteners, processed } = await processItems(options.items, 'root');

	// unlisten all events when the menu closes
	const unlistenMenuClose = await tauriEvent.listen("menu-did-close", () => {
		unlisteners.forEach((unlistener) => unlistener());
		unlisteners.length = 0;
		unlistenMenuClose();
	});

	// send the options to the plugin
	tauriApi.invoke(SHOW_COMMAND, { ...options, items: processed } as any);
}

export function onEventShowMenu(eventName: string, options: ContextMenu.EventOptions): void {
	window.addEventListener(eventName, async (e) => {
		e.preventDefault();
		
		// if options is a function, call it to get the options
		if (typeof options === 'function') {
			options = await options(e as MouseEvent);
		}

		await showMenu(options);
	});
}
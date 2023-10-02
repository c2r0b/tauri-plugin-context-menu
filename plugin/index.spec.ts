import * as tauriApi from '@tauri-apps/api';
import * as tauriEvent from '@tauri-apps/api/event';
import * as tauriApiPath from '@tauri-apps/api/path';
import { assetToPath, showMenu, onEventShowMenu, ContextMenu } from './index';

jest.mock('@tauri-apps/api', () => ({
	invoke: jest.fn()
}));

jest.mock('@tauri-apps/api/event', () => ({
	listen: jest.fn()
}));

jest.mock('@tauri-apps/api/path', () => ({
	resolveResource: jest.fn()
}));

describe('assetToPath', () => {
	it('calls tauriApiPath.resolveResource', async () => {
		const asset = 'testAsset';
		await assetToPath(asset);
		expect(tauriApiPath.resolveResource).toHaveBeenCalledWith(asset);
	});
});

describe('showMenu', () => {
	it('sets up event listeners for item events', async () => {
		const items = [
			{ event: jest.fn() },
			{
				event: jest.fn(),
				subitems: [
					{ event: jest.fn() }
				]
			}
		];
		await showMenu({ items });
		expect(tauriEvent.listen).toHaveBeenCalledTimes(4); // events + menu-did-close
	});

	it('invokes tauriApi with the SHOW_COMMAND', () => {
		showMenu({ items: [] });
		expect(tauriApi.invoke).toHaveBeenCalledWith(expect.stringMatching('plugin:context_menu|show_context_menu'), expect.any(Object));
	});
});

describe('onEventShowMenu', () => {
	it('sets up a window event listener', () => {
		const addEventListenerSpy = jest.spyOn(window, 'addEventListener');
		onEventShowMenu('testEvent', {} as ContextMenu.Options);
		expect(addEventListenerSpy).toHaveBeenCalledWith('testEvent', expect.any(Function));
		addEventListenerSpy.mockRestore();
	});
});
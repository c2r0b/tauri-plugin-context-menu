import * as ContextMenu from './types';
export type { ContextMenu };
export declare function assetToPath(asset: string): Promise<string>;
export declare function showMenu(options: ContextMenu.Options): Promise<void>;
export declare function onEventShowMenu(eventName: string, options: ContextMenu.EventOptions): void;

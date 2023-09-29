import type { UnlistenFn } from "@tauri-apps/api/event";
export interface Position {
    x: number;
    y: number;
    is_absolute?: boolean;
}
export interface Icon {
    path: string;
    width?: number;
    height?: number;
}
export interface Item {
    label?: string;
    disabled?: boolean;
    is_separator?: boolean;
    event?: string | Function;
    shortcut?: string;
    icon?: Icon;
    subitems?: Item[];
}
export interface Options {
    pos?: Position;
    items: Item[];
}
export interface ProcessResult {
    unlisteners: UnlistenFn[];
    processed: Item[];
}
export type EventOptionsFunction = (e?: MouseEvent) => Options | Promise<Options>;
export type EventOptions = Options | EventOptionsFunction;

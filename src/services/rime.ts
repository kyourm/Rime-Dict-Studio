import { invoke } from "@tauri-apps/api/core";

export interface BootstrapState { directory: string | null; file: string | null; dictionaries: string[] }
export interface DictionaryFile { path: string; content: string }

export const bootstrap = (): Promise<BootstrapState> => invoke("bootstrap");
export const listDictionaries = (directory: string): Promise<string[]> => invoke("list_dictionaries", { directory });
export const readDictionary = (path: string): Promise<DictionaryFile> => invoke("read_dictionary", { path });
export const rememberSelection = (directory: string, file: string): Promise<void> => invoke("remember_selection", { directory, file });
export const saveDictionary = (path: string, content: string): Promise<void> => invoke("save_dictionary", { path, content });

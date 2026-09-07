import { invoke } from "@tauri-apps/api/core";

export interface BootstrapState { file: string | null }
export interface DictionaryFile { path: string; content: string }

export const bootstrap = (): Promise<BootstrapState> => invoke("bootstrap");
export const readDictionary = (path: string): Promise<DictionaryFile> => invoke("read_dictionary", { path });
export const rememberSelection = (file: string): Promise<void> => invoke("remember_selection", { file });
export const saveDictionary = (path: string, content: string): Promise<void> => invoke("save_dictionary", { path, content });

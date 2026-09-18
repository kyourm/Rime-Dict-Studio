import { invoke } from "@tauri-apps/api/core";

export interface BootstrapState { file: string | null }
export interface DictionaryFile { path: string; content: string }
export interface DictionaryGroup { rootPath: string; files: DictionaryFile[]; warnings: string[] }
export interface DictionaryWrite { path: string; content: string }
export interface DeploymentState {
  available: boolean;
  label: "squirrel" | "weasel" | "librime" | "custom" | "unavailable";
  executable: string | null;
  arguments: string[];
  workingDirectory: string | null;
  experimental: boolean;
  custom: boolean;
}
export interface DeploymentConfig { executable: string; arguments: string[]; workingDirectory: string | null }

export const bootstrap = (): Promise<BootstrapState> => invoke("bootstrap");
export const readDictionary = (path: string): Promise<DictionaryFile> => invoke("read_dictionary", { path });
export const readDictionaryGroup = (path: string): Promise<DictionaryGroup> => invoke("read_dictionary_group", { path });
export const rememberSelection = (file: string): Promise<void> => invoke("remember_selection", { file });
export const saveDictionary = (path: string, content: string): Promise<void> => invoke("save_dictionary", { path, content });
export const saveDictionaries = (writes: DictionaryWrite[]): Promise<void> => invoke("save_dictionaries", { writes });
export const deploymentState = (userDirectory: string): Promise<DeploymentState> => invoke("deployment_state", { userDirectory });
export const saveDeploymentConfig = (config: DeploymentConfig | null): Promise<void> => invoke("save_deployment_config", { config });
export const deployRime = (userDirectory: string): Promise<void> => invoke("deploy_rime", { userDirectory });

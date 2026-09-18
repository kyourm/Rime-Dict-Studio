import { computed, reactive, ref, type Ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import { deployRime, deploymentState, saveDeploymentConfig, type DeploymentState } from "../services/rime";

type Notify = (text: string, kind?: "success" | "error") => void;

export function useRimeDeployment(currentFile: Ref<string>, notify: Notify) {
  const { t } = useI18n();
  const deployer = ref<DeploymentState | null>(null);
  const deploying = ref(false);
  const detecting = ref(false);
  const configuring = ref(false);
  const showSettings = ref(false);
  const draft = reactive({ executable: "", arguments: "" });
  const userDirectory = computed(() => currentFile.value.replace(/[\\/][^\\/]+$/, ""));

  async function refresh() {
    if (!userDirectory.value) return;
    detecting.value = true;
    try { deployer.value = await deploymentState(userDirectory.value); }
    catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
    finally { detecting.value = false; }
  }

  function openSettings() {
    draft.executable = deployer.value?.executable ?? "";
    draft.arguments = deployer.value?.arguments.join("\n") ?? "";
    showSettings.value = true;
  }

  async function chooseProgram() {
    configuring.value = true;
    try {
      const selected = await open({ directory: false, multiple: false, title: t("deploy.chooseProgram") });
      if (typeof selected === "string") draft.executable = selected;
    } finally { configuring.value = false; }
  }

  function parentPath(path: string): string | null {
    const parent = path.replace(/[\\/][^\\/]+$/, "");
    return parent === path ? null : parent;
  }

  async function saveSettings() {
    configuring.value = true;
    try {
      await saveDeploymentConfig({
        executable: draft.executable,
        arguments: draft.arguments.split("\n").map((argument) => argument.trim()).filter(Boolean),
        workingDirectory: parentPath(draft.executable),
      });
      showSettings.value = false;
      await refresh();
      notify(t("message.deploySettingsSaved"));
    } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
    finally { configuring.value = false; }
  }

  async function useAutomatic() {
    configuring.value = true;
    try {
      await saveDeploymentConfig(null);
      showSettings.value = false;
      await refresh();
      notify(t("message.deploySettingsSaved"));
    } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
    finally { configuring.value = false; }
  }

  async function deploy(): Promise<boolean> {
    if (!deployer.value?.available) { openSettings(); return false; }
    deploying.value = true;
    try {
      await deployRime(userDirectory.value);
      notify(t("message.deployed"));
      return true;
    } catch (error) {
      notify(`${t("message.deployFailed")}: ${String(error)}`, "error");
      return false;
    } finally { deploying.value = false; }
  }

  return { deployer, deploying, detecting, configuring, showSettings, draft, refresh, openSettings, chooseProgram, saveSettings, useAutomatic, deploy };
}

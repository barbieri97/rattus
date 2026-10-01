// Typed access to the Rust side (Tauri commands and the frame channel).
import { Channel, invoke } from "@tauri-apps/api/core";
import type { Command } from "../bindings/Command";
import type { ExportDataset } from "../bindings/ExportDataset";
import type { HostControl } from "../bindings/HostControl";
import type { HostStatus } from "../bindings/HostStatus";
import type { SimMessage } from "../bindings/SimMessage";

/** True when running inside the Tauri desktop shell (false in a plain browser). */
export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function subscribe(onMessage: (message: SimMessage) => void): Promise<void> {
  const channel = new Channel<SimMessage>();
  channel.onmessage = onMessage;
  await invoke("subscribe", { onMessage: channel });
}

export function simCommand(command: Command): Promise<void> {
  return invoke("sim_command", { command });
}

export function hostControl(control: HostControl): Promise<HostStatus> {
  return invoke("host_control", { control });
}

export function saveExperiment(path: string): Promise<void> {
  return invoke("save_experiment", { path });
}

export function openExperiment(path: string, randomStart: boolean): Promise<void> {
  return invoke("open_experiment", { path, randomStart });
}

export function exportCsv(path: string, dataset: ExportDataset): Promise<void> {
  return invoke("export_csv", { path, dataset });
}

/** Turns an error thrown by `invoke` into a readable message. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

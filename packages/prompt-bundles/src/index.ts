import { join, resolve } from "node:path";

export interface PromptModeDoc {
  id: string;
  label: string;
  relativePath: string;
}

interface PromptProfileBlueprint {
  id: string;
  name: string;
  description?: string;
  agentsDoc: string;
  modes: PromptModeDoc[];
}

export interface PromptProfileDescriptor extends PromptProfileBlueprint {}

const PROFILE_BLUEPRINTS: PromptProfileBlueprint[] = [
  {
    id: "asv-orchestrator",
    name: "ASV Orchestrator",
    description:
      "Agents/Modes stack tuned for BOOT / ORCHESTRATOR / WORKER / DOC_SCRIBE flows.",
    modes: [
      { id: "BOOT", label: "Boot Mode", relativePath: "MODES/BOOT.md" },
      {
        id: "ORCHESTRATOR",
        label: "Orchestrator Mode",
        relativePath: "MODES/ORCHESTRATOR.md"
      },
      { id: "WORKER", label: "Worker Mode", relativePath: "MODES/WORKER.md" },
      {
        id: "DOC_SCRIBE",
        label: "Doc Scribe Mode",
        relativePath: "MODES/DOC_SCRIBE.md"
      }
    ],
    agentsDoc: "AGENTS.md"
  }
];

export interface ResolveOptions {
  baseDir?: string;
}

export function listPromptProfiles(
  options: ResolveOptions = {}
): PromptProfileDescriptor[] {
  const baseDir = resolve(options.baseDir || "prompts/profiles");
  return PROFILE_BLUEPRINTS.map((profile) => ({
    ...profile,
    agentsDoc: join(baseDir, profile.id, profile.agentsDoc),
    modes: profile.modes.map((mode) => ({
      ...mode,
      relativePath: join(baseDir, profile.id, mode.relativePath)
    }))
  }));
}

export function getPromptProfile(
  id: string,
  options: ResolveOptions = {}
): PromptProfileDescriptor | undefined {
  return listPromptProfiles(options).find((profile) => profile.id === id);
}
